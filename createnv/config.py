from dataclasses import dataclass
from random import choice, randint
from typing import Iterable, Mapping, Optional, Union

from typer import prompt


@dataclass
class Config:
    name: str
    human_name: Optional[str] = None
    default: Optional[str] = None

    def __str__(self) -> str:
        return self.human_name or self.name

    def __call__(self, use_default: bool = False) -> str:
        if self.default and use_default:
            return self.default
        return prompt(str(self), default=self.default)


@dataclass
class RandomConfig:
    name: str
    allowed_chars: str
    human_name: Optional[str] = None
    length: Optional[int] = None

    def __str__(self) -> str:
        return self.human_name or self.name

    @property
    def default(self) -> str:
        length = self.length or randint(64, 128)
        chars = (choice(self.allowed_chars) for _ in range(length))
        return "".join(chars)

    def __call__(self, use_default: bool = False) -> str:
        if use_default:
            return self.default

        return prompt(str(self), default=self.default)


@dataclass
class AutoConfig:
    """Config generated within a Group, using other Config values. The `value`
    format method is called used `arguments`, so it expects the curly-braces
    syntax with ordered arguments."""

    name: str
    value: str

    def __call__(self, settings: Mapping[str, str]) -> Mapping[str, str]:
        value = self.value.format(**settings)
        return {self.name: value}


@dataclass
class Group:
    title: str
    configs: Iterable[Union[Config, RandomConfig]]
    description: Optional[str] = None
    auto_config: Optional[AutoConfig] = None

    def __str__(self):
        contents = ("", self.title, f"({self.description})")
        return "\n".join(contents if self.description else contents[:-1])

    def should_echo(self, use_default: bool = False) -> bool:
        if not use_default:
            return True

        return not all(c.default for c in self.configs)

    def __call__(self, use_default: bool = False) -> Mapping[str, str]:
        settings = {c.name: c(use_default) for c in self.configs}

        if self.auto_config:
            auto_settings = self.auto_config(settings)
            settings.update(auto_settings)

        return settings
