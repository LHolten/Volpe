from lark import Tree


class TypeTree(Tree):
    return_type = None

    def _pretty_label(self):
        if self.return_type is not None:
            return f'{self.data}: {self.return_type}'
        return self.data
