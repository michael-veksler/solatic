#!/usr/bin/env python3
""" Generate AllDiff CNF"""


from abc import ABC, abstractmethod
import argparse
from dataclasses import dataclass
from enum import StrEnum
from io import StringIO
import itertools


class IntFormat(StrEnum):
    """The encoding to use for int representation"""
    BASE2 = "base2"
    ONE_HOT = "one-hot"
    DEFAULT = BASE2


def get_args() -> argparse.Namespace:
    """Parse the cmdline arguments"""
    parser = argparse.ArgumentParser(description="Construct a CNF for all-diff",
                                     formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument("--int-bits", type=int, required=True,
                        help="How many bits per integer")
    parser.add_argument("--n-ints", type=int, required=True,
                        help="How many integers to have in all-diff")

    parser.add_argument("--domain-size", type=int,
                        help="The number of possible integer values to choose from")

    parser.add_argument("--int-format", choices=list(IntFormat), type=IntFormat, default=IntFormat.DEFAULT,
                        help=("Choose a representation for integers.\n"
                              "Base2 is the standard binary representation in computers.\n"
                              "One-hot is when for domain [0..N-1] there are bits b[0]..b[N-1] where exactly one bit "
                              "is 1, so that b[x] == 1 means that integer x is the solution."))

    args = parser.parse_args()
    args.prog = parser.prog
    return args

class CnfDb:
    """Class containing current CNF status"""
    _num_vars: int = 0
    _num_clauses: int = 0
    _header_comments: list[str]
    _out_stream: StringIO

    def __init__(self):
        self._out_stream = StringIO()
        self._header_comments = []

    def add_var(self) -> int:
        """Add a variable to the CNF"""
        self._num_vars += 1
        return self._num_vars

    def add_clause(self, clause: list[int]) -> None:
        """Add the clause to the CNF

           Each list entry `lit` is:
            - lit>0 for var(lit) == true
            - lit<0 for var(abs(lit)) == false
            - 0: Illegal value, should not be in here
        """
        assert all(lit != 0 for lit in clause)
        self._num_clauses += 1
        print(' '.join(f'{lit}' for lit in clause), 0,
              file=self._out_stream)

    def add_comment(self, comment: str) -> None:
        """Add comment line(s)"""
        for line in comment.splitlines():
            print("c", line,
                  file=self._out_stream)

    def add_header_comment(self, comment: str) -> None:
        """Register the comments that are in the header of the DIMACS CNF file"""
        self._header_comments.extend(comment.splitlines())

    def add_separators(self, count: int=1) -> None:
        """Add line separators in the final CNF, when supported"""
        print('\n' * count, file=self._out_stream)

    def print(self) -> None:
        """Print the CNF to stdout in DIMACS format"""
        for line in self._header_comments:
            print(f"c {line}")
        print(f"p {self._num_vars} {self._num_clauses}")
        print(self._out_stream.getvalue(), end='')

    def vars_to_str(self, bool_vars: list[int]) -> str:
        """Get a string representation for variables
        For example vars_to_str([1,2]) -> "V1, V2"
        """
        return ', '.join(f'V{bool_var}' for bool_var in bool_vars)

@dataclass
class IntSpec:
    """How integers are represented including all parameters"""
    int_format: IntFormat
    int_bits: int
    domain_size: int|None

class IntDb:
    """Class responsible for mapping from integer variables to binary variables"""
    int_spec: IntSpec
    n_ints: int
    cnf_db: CnfDb
    int_representation: IntRepresentation
    _int_var_as_bools: dict[int, list[int]]

    def __init__(self, int_spec: IntSpec, n_ints: int, cnf_db: CnfDb):
        self.int_spec = int_spec
        self.n_ints = n_ints
        self.cnf_db = cnf_db
        self._int_var_as_bools = {}

        representations: dict[IntFormat, type[IntRepresentation]] = {
            IntFormat.BASE2: Base2IntRepresentation,
            IntFormat.ONE_HOT: OneHotRepresentation
        }
        self.int_representation = representations[self.int_spec.int_format]()


    def get(self, number: int) -> list[int]:
        """Get the list of boolean variables that represent this number, possibly creating them"""
        result = self._int_var_as_bools.get(number)
        if result is None:
            result =[self.cnf_db.add_var() for _ in range(self.int_spec.int_bits)]
            self.cnf_db.add_comment(f"int({number}) V{result[0]}..={result[-1]}")
            self._int_var_as_bools[number] = result
            self.int_representation.build_domain_constraint(self, result)
        return result


    def add_cnf_header(self, cnf_db: CnfDb, prog: str) -> None:
        """Add the integer info the the CNF header"""
        spec = self.int_spec
        cnf_db.add_header_comment(f"CNF for all-diff with {self.n_ints} integers with {spec.int_bits} bits each")
        if_domain = '' if self.int_spec.domain_size is None else f' --domain-size={spec.domain_size}'
        cnf_db.add_header_comment(f"generated by {prog} --int-bits={spec.int_bits} --n-ints={self.n_ints}{if_domain}")

class ConstraintDb: # pylint: disable=too-few-public-methods
    """Manage constraints"""
    int_db: IntDb
    _eq_bool_vars:  dict[tuple[int, int], int]

    def __init__(self, int_db: IntDb):
        self.int_db = int_db
        self._eq_bool_vars = {}

    @property
    def cnf_db(self) -> CnfDb:
        """Get the CNF database associated with these constraints"""
        return self.int_db.cnf_db
    @property
    def int_spec(self) -> IntSpec:
        """Get integer representation parameters"""
        return self.int_db.int_spec

    def eval_bool_eq(self, first: int, second: int) -> int:
        """Return a bool var that will hold: var <==> (first == second)"""
        first, second = min(first, second), max(first, second)
        if first == second:
            first, second = 1, 1

        eq_var = self._eq_bool_vars.get((first, second))
        if eq_var is not None:
            return eq_var
        eq_var = self.cnf_db.add_var()
        self._eq_bool_vars[(first, second)] = eq_var
        self.cnf_db.add_comment(f'(V{first} == V{second}) <==> V{eq_var}')

        self._add_iff_eq_constraint(first, second, eq_var)

        return eq_var

    def _add_iff_eq_constraint(self, first: int, second: int, eq_var: int):
        """
            Equality constraint
              eq_var <==> (first == second)
            is split in 2:
              1. (first == second) -> eq_var
                 And is handled in steps:
                 - (first != second) || eq_var
                 - ((first -> !second) || eq_var) && ((!first -> second) || eq_var)
                 Which gives clauses: (-first, -second, eq_var);  (first, second, eq_var)
              - (first != second) -> !eq_var
                 And is handled in steps:
                 - (first == second) || !eq_var
                 - ((first -> second) || !eq_var) &&
                   ((first -> second) || !eq_var)
                 Which gives clauses: (-first, second, -eq_var);  (first, -second, -eq_var)
        """
        if first == second:
            self.cnf_db.add_clause([eq_var])
        else:
            self.cnf_db.add_clause([-first, -second, eq_var])
            self.cnf_db.add_clause([first, second, eq_var])
            self.cnf_db.add_clause([-first, second, -eq_var])
            self.cnf_db.add_clause([first, -second, -eq_var])
        self.cnf_db.add_separators()


class IntRepresentation(ABC):
    """A strategy object that manages int handling under a specific int representation"""
    @abstractmethod
    def build_domain_constraint(self, int_db: IntDb, bool_vars: list[int]) -> None:
        """Build for integers to be legal under the representation"""

    @abstractmethod
    def build_ne_constraint(self, constraint_db: ConstraintDb, first_int: int, second_int: int):
        """Constraint two integer variables to be different"""


class Base2IntRepresentation(IntRepresentation):
    """The strategy for handling base-2 (i.e. binary) representation"""
    def build_domain_constraint(self, int_db: IntDb, bool_vars: list[int]) -> None:
        """Make sure this integer variable fit the domain size, i.e., value(var) < self.int_spec.domain_size

        Args:
            bool_vars: standard base2 binary representation, where bool_vars[i] is the i-th bit.
        """
        spec = int_db.int_spec
        if spec.domain_size is None:
            return
        if spec.domain_size >= 2**len(bool_vars):
            return
        max_bits = [((spec.domain_size-1) >> bit_index) & 1 for bit_index, _ in enumerate(bool_vars)]
        cnf_db = int_db.cnf_db
        cnf_db.add_comment(f"({cnf_db.vars_to_str(bool_vars)}) <= {spec.domain_size-1}")

        # 0 prefix (MSB) will constrain all these MSB to be 0
        while max_bits and max_bits[-1] == 0:
            cnf_db.add_clause([max_bits.pop()])
            bool_vars.pop()

        # 1 suffix (LSB) means all these LSB are unconstrained
        while max_bits and max_bits[0] == 1:
            bool_vars.pop(0)
            max_bits.pop(0)
        if not max_bits:
            return

        if max_bits != [0]+ [1] * (len(max_bits)-1):
            raise ValueError("supporting domain sizes up to 2**bits - 2")

        cnf_db.add_clause([-var for var in bool_vars])

    def build_ne_constraint(self, constraint_db: ConstraintDb, first_int: int, second_int: int):
        """Create a single != constraint, relying on self.bool_constraint_db correctness"""
        int_db = constraint_db.int_db
        first_bools = int_db.get(first_int)
        second_bools = int_db.get(second_int)
        cnf_db = int_db.cnf_db
        cnf_db.add_comment(f"(int({first_int}) != int({second_int})) :: "
                           f"({cnf_db.vars_to_str(first_bools)}) != ({cnf_db.vars_to_str(second_bools)})")

        at_least_one_should_be_false: list[int] = [
            constraint_db.eval_bool_eq(first_bool, second_bool)
            for first_bool, second_bool in zip(first_bools, second_bools)]

        cnf_db.add_clause([-var for var in at_least_one_should_be_false])

class OneHotRepresentation(IntRepresentation):
    """The strategy for handling one-hot representation

    In one-hot, each Boolean variable represent a possible value.
    If an integer variable x has the domain of {1,2,3}, then the boolean variables v1, v2, v3
    may represent x, such that if v2=1 then x=2, and if v1=1 then x=1.
    This means that a general constraints are:
      - At most one of them is set: (!v1 || !v2), (!v1 || !v3), (!v2 || !v3)
      - At least one of them is set: (v1 || v2 || v3)

    Number of bits also define the domain size.
    """
    def build_domain_constraint(self, int_db: IntDb, bool_vars: list[int]) -> None:
        """Make sure this integer conform to the one-hot structure.

        Args:
            bool_vars: standard one-hot binary representation (see class docstring).
        """
        spec = int_db.int_spec
        if spec.domain_size is not None:
            raise RuntimeError("You can't specify --domain-size for --int-format one-hot")
        cnf_db = int_db.cnf_db
        cnf_db.add_comment(f"({cnf_db.vars_to_str(bool_vars)}) valid one-hot")
        for first_index, first_var in enumerate(bool_vars[:-1]):
            for second_var in itertools.islice(bool_vars, first_index+1, len(bool_vars)):
                cnf_db.add_clause([-first_var, -second_var])

        cnf_db.add_clause(bool_vars)

    def build_ne_constraint(self, constraint_db: ConstraintDb, first_int: int, second_int: int):
        """Create a single != constraint, relying on self.bool_constraint_db correctness"""
        int_db = constraint_db.int_db
        first_bools = int_db.get(first_int)
        second_bools = int_db.get(second_int)
        cnf_db = int_db.cnf_db
        cnf_db.add_comment(f"(int({first_int}) != int({second_int})) :: "
                           f"({cnf_db.vars_to_str(first_bools)}) != ({cnf_db.vars_to_str(second_bools)})")

        for first_bool, second_bool in zip(first_bools, second_bools):
            cnf_db.add_clause([-first_bool, -second_bool])


class AllDiffBuilder:
    """Main class to build a CNF for all-diff"""

    def __init__(self, prog: str, int_spec: IntSpec, n_ints: int) -> None:
        self.cnf_db = CnfDb()
        self.int_db = IntDb(int_spec=int_spec,
                            n_ints=n_ints,
                            cnf_db=self.cnf_db)
        self.constraint_db = ConstraintDb(self.int_db)
        self.prog = prog

    def build_cnf(self) -> None:
        """Construct a CNF in DIMACS format and output to stdout"""

        for i in range(self.int_db.n_ints):
            self.int_db.get(i)  # For DIMACS clarity, create all the integers in advance
        self.build_all_diff_constraints()

        self.int_db.add_cnf_header(self.cnf_db, self.prog)
        self.cnf_db.print()

    def build_all_diff_constraints(self) -> None:
        """Build all the inequality constraints of all_diff"""
        for first_int in range(self.int_db.n_ints):
            for second_int in range(first_int +1, self.int_db.n_ints):
                self.int_db.int_representation.build_ne_constraint(self.constraint_db, first_int, second_int)

def main():
    """The main of this script"""
    args = get_args()
    int_spec = IntSpec(int_format=args.int_format, int_bits=args.int_bits, domain_size=args.domain_size)
    builder = AllDiffBuilder(prog=args.prog, int_spec=int_spec, n_ints=args.n_ints)
    builder.build_cnf()

if __name__ == "__main__":
    main()
