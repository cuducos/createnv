from pathlib import Path

from createnv.config import Config, Group
from createnv.generator import Generator
from createnv.parser import ParserError


def test_can_write_to_non_existent_path_without_overwrite(mocker):
    exists = mocker.patch.object(Path, "exists")
    exists.return_value = False
    generator = Generator(Path(".env"), mocker.Mock(), False, False)
    assert generator.can_write_to_path()


def test_can_write_to_non_existent_path_with_overwrite(mocker):
    exists = mocker.patch.object(Path, "exists")
    exists.return_value = False
    generator = Generator(Path(".env"), mocker.Mock(), True, False)
    assert generator.can_write_to_path()


def test_can_write_to_existent_path_with_overwrite(mocker):
    exists = mocker.patch.object(Path, "exists")
    exists.return_value = False
    generator = Generator(Path(".env"), mocker.Mock(), True, False)
    assert generator.can_write_to_path()


def test_can_write_to_existent_path_manually_confirming_overwrite(mocker):
    exists = mocker.patch.object(Path, "exists")
    exists.return_value = True
    confirm = mocker.patch("createnv.generator.confirm")
    confirm.return_value = True
    warning = mocker.patch("createnv.generator.warning")
    generator = Generator(Path(".env"), mocker.Mock(), False, False)
    assert generator.can_write_to_path()
    warning.assert_called_once_with("There is an existing .env file.")
    confirm.called_once_with("Do you want to overwrite it?")


def test_can_write_to_existent_path_without_manually_confirming_overwrite(mocker):
    exists = mocker.patch.object(Path, "exists")
    exists.return_value = True
    confirm = mocker.patch("createnv.generator.confirm")
    confirm.return_value = False
    warning = mocker.patch("createnv.generator.warning")
    generator = Generator(Path(".env"), mocker.Mock(), False, False)
    assert not generator.can_write_to_path()
    warning.assert_called_once_with("There is an existing .env file.")
    confirm.assert_called_once_with("Do you want to overwrite it?")


def test_contents(mocker):
    group = mocker.patch.object(Group, "__call__")
    group.side_effect = ({"NAME": "Cuducos", "PERIOD": "morning"}, {"DEBUG": "True"})
    should_echo = mocker.patch.object(Group, "should_echo")
    should_echo.side_effect = (True, False)
    echo = mocker.patch("createnv.generator.echo")
    generator = Generator(Path(".env"), mocker.Mock(), False, False)
    settings = (
        Group("Greeting", (Config("NAME"), Config("Period")), "Say hi!"),
        Group("Environment", (Config("DEBUG"))),
    )
    assert tuple(generator.contents(settings)) == (
        "# Greeting",
        "# Say hi!",
        "NAME=Cuducos",
        "PERIOD=morning",
        "",
        "# Environment",
        "DEBUG=True",
        "",
    )
    echo.assert_called_once_with("\nGreeting\n(Say hi!)")


def test_call_prints_errors_from_parser(mocker):
    error = mocker.patch("createnv.generator.error")
    write_text = mocker.patch.object(Path, "write_text")
    success = mocker.patch("createnv.generator.success")
    parser = mocker.Mock()
    parser.side_effect = ParserError("oops")
    generator = Generator(Path(".env"), parser, False, False)
    generator()
    parser.assert_called_once_with()
    error.assert_called_once_with("oops")
    write_text.assert_not_called()
    success.assert_not_called()


def test_call_stops_when_it_cannot_write_to_target(mocker):
    error = mocker.patch("createnv.generator.error")
    write_text = mocker.patch.object(Path, "write_text")
    success = mocker.patch("createnv.generator.success")
    can_write_to_path = mocker.patch.object(Generator, "can_write_to_path")
    parser = mocker.Mock()
    can_write_to_path.return_value = False
    generator = Generator(Path(".env"), parser, False, False)
    generator()
    parser.assert_called_once_with()
    error.assert_not_called()
    write_text.assert_not_called()
    success.assert_not_called()


def test_call(mocker):
    error = mocker.patch("createnv.generator.error")
    write_text = mocker.patch.object(Path, "write_text")
    success = mocker.patch("createnv.generator.success")
    can_write_to_path = mocker.patch.object(Generator, "can_write_to_path")
    can_write_to_path.return_value = True
    contents = mocker.patch.object(Generator, "contents")
    contents.return_value = ("Hell yeah!", "This is awesome")
    parser = mocker.Mock()
    generator = Generator(Path(".env"), parser, False, False)
    generator()
    error.assert_not_called()
    write_text.assert_called_once_with("Hell yeah!\nThis is awesome")
    success.assert_called_once_with(".env created!")
