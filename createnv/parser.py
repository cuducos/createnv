from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from re import match, search
from typing import Iterable, Iterator, List, Optional, Union

from createnv.config import AutoConfig, Config, Group, RandomConfig


@dataclass
class Line:
    number: int
    contents: str

    def cleaned(self) -> str:
        return self.contents.strip()

    def is_empty(self) -> bool:
        return not bool(self.cleaned())

    def is_comment(self) -> bool:
        return bool(self.cleaned().startswith("#"))


@dataclass
class Block:
    lines: List[Line] = field(default_factory=list)

    def __add__(self, line: Line) -> Block:
        self.lines.append(line)
        return self

    def __iter__(self) -> Iterator[Line]:
        yield from self.lines

    def is_empty(self) -> bool:
        return not bool(self.lines)


class ParserError(Exception):
    def __init__(self, *args, **kwargs):
        if isinstance(args[0], Line):
            line, text, *new_args = args
            return super().__init__(self.message(line, text), *new_args, **kwargs)
        return super().__init__(*args, **kwargs)

    @staticmethod
    def message(line: Line, text: str) -> str:
        message = (
            "",
            f"==> Parsing error at line {line.number}:",
            f"    {text}",
            "",
            f"    The content of the line {line.number} is:",
            f"    {line.contents}",
            "",
        )
        return "\n".join(message)


@dataclass
class Parser:
    source: Path
    chars_for_random_string: str

    TITLE: str = r"^# (?P<title>.+)$"
    CONFIG: str = r"^(?P<name>[A-Z_0-9]+)=(?P<value>.+)?"
    AUTO_CONFIG_VALUE: str = r"{[A-Z_0-0]+}"
    RANDOM_VALUE: str = r"<random(:(?P<length>\d+))?>"
    INLINE_COMMENT: str = "  # "

    def blocks(self) -> Iterator[Block]:
        block = Block()
        for values in enumerate(self.source.open(), 1):
            line = Line(*values)
            if line.is_empty():
                if block.is_empty():
                    continue

                yield block
                block = Block()

            else:
                block += line

        if not block.is_empty():
            yield block

    def parse_title(self, line: Line) -> str:
        matches = match(self.TITLE, line.cleaned())
        if not matches:
            message = (
                f"This is the first line of a block in {self.source}. A block "
                "is a group of lines separated from others by one (or more) "
                "empty line(s). The first line of a block is expected to be a "
                "title, that is to say, to start with `# `, the remaining "
                "text is considered the title of this block. This lines "
                "does not match this pattern."
            )
            raise ParserError(line, message)

        return matches.group("title")

    def parse_config(self, line: Line) -> Union[Config, AutoConfig, RandomConfig]:
        matches = match(self.CONFIG, line.cleaned())
        if not matches:
            message = (
                "This line was expected to be a config variable. The format "
                "should be a name using capital ASCII letters, digits or "
                "underscore, followed by an equal sign. This line does not "
                "match this expected pattern."
            )
            raise ParserError(line, message)

        name, value, human = matches.group("name"), matches.group("value"), None
        if value is not None and self.INLINE_COMMENT in value:
            value, human = value.rsplit(self.INLINE_COMMENT, maxsplit=1)

        if search(self.AUTO_CONFIG_VALUE, value or ""):
            return AutoConfig(name, value)

        random_match = match(self.RANDOM_VALUE, value or "")
        if random_match:
            length = random_match.group("length")
            return RandomConfig(
                name=name,
                allowed_chars=self.chars_for_random_string,
                human_name=human or None,
                length=int(length) if length else None,
            )

        return Config(name, human or None, value or None)

    def parse_description_or_config(
        self, line: Line
    ) -> Union[str, Config, AutoConfig, RandomConfig]:
        method = self.parse_title if line.is_comment() else self.parse_config
        try:
            result = method(line)
        except ParserError:
            message = (
                f"This is the second line of a block in {self.source}. A "
                "block is a group of lines separated from others by one (or "
                "more) empty line(s). The second line of a block is expected "
                "to be a description of that block or a config variable. The "
                "description line should start `# `, and the remaining text "
                "is considered the description of this block. A config "
                "variable line should start with a name in uppercase, no "
                "spaces, followed by an equal sign. This lines does not match "
                "this expected patterns."
            )
            raise ParserError(line, message)

        return result

    def parse(self, block: Block) -> Group:
        _title, _description_or_config, *_configs = block

        title: str = self.parse_title(_title)
        configs: List[Union[Config, RandomConfig]] = []
        description: Optional[str] = None
        auto_config: Optional[AutoConfig] = None

        parsed = self.parse_description_or_config(_description_or_config)
        if isinstance(parsed, str):
            description = parsed
        elif isinstance(parsed, AutoConfig):
            auto_config = parsed
        else:
            configs.append(parsed)

        for line in _configs:
            parsed = self.parse_config(line)
            if isinstance(parsed, (Config, RandomConfig)):
                configs.append(parsed)
            elif isinstance(parsed, AutoConfig) and not auto_config:
                auto_config = parsed

        return Group(
            title=title,
            configs=configs,
            description=description,
            auto_config=auto_config,
        )

    def __call__(self) -> Iterable[Group]:
        if not self.source.exists():
            raise ParserError(f"{self.source} does not exist.")

        if not self.source.is_file():
            raise ParserError(f"{self.source} is not a file.")

        return tuple(self.parse(block) for block in self.blocks())
