from pathlib import Path

import pytest  # type: ignore

from createnv.cli import RANDOM_CHARS
from createnv.config import AutoConfig, Config, Group, RandomConfig
from createnv.parser import Block, Line, Parser, ParserError


def test_block(mocker):
    path_open = mocker.patch.object(Path, "open")
    path_open.return_value = (
        "\t",
        "# Title",
        "# Description",
        "VARIABLE=",
        "",
        "# Another title",
        "ANOTHER_VARIABLE=",
        "",
    )
    expected = (
        Block([Line(2, "# Title"), Line(3, "# Description"), Line(4, "VARIABLE=")]),
        Block([Line(6, "# Another title"), Line(7, "ANOTHER_VARIABLE=")]),
    )
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    assert tuple(parser.blocks()) == expected


def test_parse_title():
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    assert parser.parse_title(Line(2, "# Title")) == "Title"
    with pytest.raises(ParserError):
        parser.parse_title(Line(2, "VARIABLE="))


def test_parse_config_with_config_lines():
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    values = (
        (Line(4, "VARIABLE="), Config("VARIABLE")),
        (Line(4, "VARIABLE=  # Variable"), Config("VARIABLE", "Variable")),
        (Line(4, "VARIABLE=42"), Config("VARIABLE", default="42")),
        (Line(4, "VARIABLE=42  # Variable"), Config("VARIABLE", "Variable", "42")),
    )
    for line, expected in values:
        assert parser.parse_config(line) == expected


def test_parse_config_with_auto_config_line():
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    line = Line(4, "VARIABLE=Hello, {NAME}!")
    assert parser.parse_config(line) == AutoConfig("VARIABLE", "Hello, {NAME}!")


def test_parse_config_with_random_config_lines():
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    lines = (Line(4, "VARIABLE=<random>"), Line(4, "VARIABLE=<random:42>  # Variable"))
    expected = (
        RandomConfig("VARIABLE", parser.chars_for_random_string),
        RandomConfig("VARIABLE", parser.chars_for_random_string, "Variable", length=42),
    )
    for line, expected in zip(lines, expected):
        assert parser.parse_config(line) == expected


def test_parse_config_with_invalid_line():
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    with pytest.raises(ParserError):
        parser.parse_config(Line(2, "# Title"))


def test_parse_description_or_config_with_description(mocker):
    description = Line(3, "# Here comes a description")
    parse_title = mocker.patch.object(Parser, "parse_title")
    parse_config = mocker.patch.object(Parser, "parse_config")
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    parser.parse_description_or_config(description)
    parse_title.assert_called_once_with(description)
    parse_config.asseert_not_called()


def test_parse_description_or_config_with_config(mocker):
    config = Line(4, "VARIABLE=")
    parse_title = mocker.patch.object(Parser, "parse_title")
    parse_config = mocker.patch.object(Parser, "parse_config")
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    parser.parse_description_or_config(config)
    parse_title.assert_not_called()
    parse_config.assert_called_once_with(config)


def test_parse_description_or_config_with_invalid_line():
    nothing = Line(1, "\t")
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    with pytest.raises(ParserError):
        parser.parse_description_or_config(nothing)


def test_call_raises_error_without_source():
    fixture = Path() / "tests" / ".env.sample.that.does.not.exist"
    parser = Parser(fixture, RANDOM_CHARS)
    with pytest.raises(ParserError):
        parser()


def test_call_raises_error_if_source_is_a_directory():
    fixture = Path() / "tests"
    parser = Parser(fixture, RANDOM_CHARS)
    with pytest.raises(ParserError):
        parser()


def test_parser():
    fixture = Path() / "tests" / ".env.sample"
    parser = Parser(fixture, RANDOM_CHARS)
    expected = (
        Group(
            title="This is the title",
            description="(Here comes details to make the interface more user-friendly)",
            configs=[
                Config("MY_FIRST_VARIABLE"),
                Config("MY_SECOND_VARIABLE", default="42"),
                Config(
                    "MY_THIRD_VARIABLE", human_name="My third variable", default="42"
                ),
            ],
        ),
        Group(
            title="This block has no description",
            configs=[Config("SHOULD_I_DO_THAT", default="False")],
        ),
        Group(
            title="This block uses the auto-config and the random features",
            configs=[
                Config("NAME", default="Cuducos"),
                Config("PERIOD", default="morning"),
                Config("THIS_IS_NOT_USED_IN_AUTO_CONFIG", default="ok?"),
                RandomConfig("I_HAVE_A_SECRET", RANDOM_CHARS),
                RandomConfig("I_HAVE_A_PIN_SECRET", RANDOM_CHARS, length=4),
            ],
            auto_config=AutoConfig("GREETINGS", "Good {PERIOD}, {NAME}!"),
        ),
        Group(
            title="Test auto-config as first line",
            configs=[
                Config("NAME", default="Cuducos"),
                Config("PERIOD", default="morning"),
            ],
            auto_config=AutoConfig("GREETINGS", "Good {PERIOD}, {NAME}!"),
        ),
    )
    result = parser()
    assert result == expected
