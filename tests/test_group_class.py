from unittest.mock import call

from createnv.config import AutoConfig, Config, Group


def test_complete_str():
    group = Group("Hell yeah!", (Config("DEBUG"),), "This is awesome!")
    assert str(group) == "\nHell yeah!\n(This is awesome!)"


def test_str_without_description():
    group = Group("Hell yeah!", (Config("DEBUG"),))
    assert str(group) == "\nHell yeah!"


def test_should_echo_without_default_but_trying_to_use_default():
    group = Group("Hell yeah!", (Config("DEBUG"),))
    assert group.should_echo(True)


def test_should_echo_without_default_and_not_trying_to_use_default():
    group = Group("Hell yeah!", (Config("DEBUG"),))
    assert group.should_echo()


def test_should_echo_with_default_and_trying_to_use_default():
    group = Group("Hell yeah!", (Config("DEBUG", default="42"),))
    assert not group.should_echo(True)


def test_should_echo_with_default_and_not_trying_to_use_default():
    group = Group("Hell yeah!", (Config("DEBUG", default="42"),))
    assert group.should_echo()


def test_call_without_auto_config_without_use_default(mocker):
    config = mocker.patch.object(Config, "__call__")
    config.side_effect = ("True", "localhost")
    group = Group("Hell yeah!", (Config("DEBUG"), Config("ALLOWED_HOSTS")))
    assert group() == {"DEBUG": "True", "ALLOWED_HOSTS": "localhost"}
    assert config.call_count == 2
    config.assert_has_calls((call(False), call(False)))


def test_call_without_auto_config_with_use_default(mocker):
    config = mocker.patch.object(Config, "__call__")
    config.side_effect = ("True", "localhost")
    group = Group("Hell yeah!", (Config("DEBUG"), Config("ALLOWED_HOSTS")))
    assert group(True) == {"DEBUG": "True", "ALLOWED_HOSTS": "localhost"}
    assert config.call_count == 2
    config.assert_has_calls((call(True), call(True)))


def test_call_with_auto_config(mocker):
    config = mocker.patch.object(Config, "__call__")
    config.side_effect = ("morning", "Cuducos")
    auto_config = mocker.patch.object(AutoConfig, "__call__")
    auto_config.return_value = {"GREETING": "Good morning, Cuducos!"}
    group = Group(
        "Hell yeah!",
        (Config("PERIOD"), Config("NAME")),
        "This is pretty cool.",
        AutoConfig("GREETING", "Good {PERIOD}, {NAME}!"),
    )
    assert group() == {
        "NAME": "Cuducos",
        "PERIOD": "morning",
        "GREETING": "Good morning, Cuducos!",
    }
