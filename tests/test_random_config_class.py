from createnv.config import RandomConfig


def test_str():
    assert str(RandomConfig("SECRET_KEY", "ab")) == "SECRET_KEY"
    assert str(RandomConfig("SECRET_KEY", "ab", "Secret key")) == "Secret key"


def test_default():
    config = RandomConfig("SECRET_KEY", "ab", "Secret key", 2)
    assert config.default in {"aa", "ab", "ba", "bb"}


def test_call_without_default(mocker):
    prompt = mocker.patch("createnv.config.prompt")
    prompt.return_value = "42"
    config = RandomConfig("SECRET_KEY", "a", "Secret key", 3)
    assert config() == "42"
    prompt.assert_called_once_with("Secret key", default="aaa")


def test_call_with_default(mocker):
    prompt = mocker.patch("createnv.config.prompt")
    config = RandomConfig("SECRET_KEY", "a", "Secret key", 3)
    assert config(True) == "aaa"
    prompt.assert_not_called()
