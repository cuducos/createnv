from createnv.parser import Line


def test_cleaned():
    assert Line(42, "  \tyay\n  ").cleaned() == "yay"


def test_is_empty():
    assert Line(42, "").is_empty()
    assert Line(42, " ").is_empty()
    assert Line(42, "\t").is_empty()
    assert Line(42, "\n\t").is_empty()
    assert not Line(42, "  \tyay\n  ").is_empty()


def test_is_comment():
    assert Line(42, "# Hell yeah!").is_comment()
    assert Line(42, "\t# Hell yeah!").is_comment()
    assert Line(42, "    # Hell yeah!").is_comment()
    assert not Line(42, "").is_comment()
    assert not Line(42, "NAME=CUDUCOS").is_comment()
