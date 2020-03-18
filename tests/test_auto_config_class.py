from createnv.config import AutoConfig


def test_call():
    settings = {"NAME": "Cuducos", "PERIOD": "morning"}
    config = AutoConfig("GREETING", "Good {PERIOD}, {NAME}!")
    assert config(settings) == {"GREETING": "Good morning, Cuducos!"}
