from createnv.config import Config


def test_str():
    human, bot = Config("DEBUG", "Debug mode"), Config("DEBUG")
    assert str(human) == "Debug mode"
    assert str(bot) == "DEBUG"


def test_call_with_default_using_default(mocker):
    prompt = mocker.patch("createnv.config.prompt")
    config = Config("DEBUG", "Debug mode", "True")
    assert config(True) == "True"
    prompt.assert_not_called()


def test_call_with_default_but_not_using_default(mocker):
    prompt = mocker.patch("createnv.config.prompt")
    prompt.return_value = "42"
    config = Config("DEBUG", "Debug mode", "True")
    assert config() == "42"
    prompt.assert_called_once_with(str(config), default="True")


def test_call_without_default_trying_to_use_default(mocker):
    prompt = mocker.patch("createnv.config.prompt")
    prompt.return_value = "42"
    config = Config("DEBUG", "Debug mode")
    assert config(True) == "42"
    prompt.assert_called_once_with(str(config), default=None)


def test_call_without_default_without_trying_to_use_default(mocker):
    prompt = mocker.patch("createnv.config.prompt")
    prompt.return_value = "42"
    config = Config("DEBUG", "Debug mode")
    assert config() == "42"
    prompt.assert_called_once_with(str(config), default=None)
