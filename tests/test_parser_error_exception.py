from createnv.parser import Line, ParserError


def test_regular_call():
    exception = ParserError("oops")
    assert str(exception) == "oops"


def test_call_with_line():
    exception = ParserError(Line(42, "yay"), "oops")
    assert str(exception) == (
        "\n"
        "==> Parsing error at line 42:\n"
        "    oops\n"
        "\n"
        "    The content of the line 42 is:\n"
        "    yay\n"
    )
