from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Iterator

from typer import confirm

from createnv import echo, error, success, warning
from createnv.config import Group
from createnv.parser import Parser, ParserError


@dataclass
class Generator:
    path: Path
    parser: Parser
    overwrite: bool
    use_default: bool

    def can_write_to_path(self) -> bool:
        if self.overwrite or not self.path.exists():
            return True

        warning(f"There is an existing {self.path.name} file.")
        return confirm("Do you want to overwrite it?")

    def contents(self, settings: Iterable[Group]) -> Iterator[str]:
        for group in settings:
            if group.should_echo(self.use_default):
                echo(str(group))

            values = group(self.use_default)
            yield f"# {group.title}"
            if group.description:
                yield f"# {group.description}"
            yield from (f"{key}={value}" for key, value in values.items())
            yield ""

    def __call__(self) -> None:
        try:
            settings = self.parser()
        except ParserError as parser_error:
            error(str(parser_error))
            return

        if not self.can_write_to_path():
            return

        contents = "\n".join(self.contents(settings))
        self.path.write_text(contents)
        success(f"{self.path.name} created!")
