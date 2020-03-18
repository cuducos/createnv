from createnv.parser import Block, Line


def test_init():
    block = Block()
    assert block.lines == []


def test_add():
    line1, line2 = Line(1, "Hell yeah!"), Line(2, "This is awesome")
    block = Block()
    block + line1
    block += line2
    assert block.lines == [line1, line2]


def test_iter():
    lines = (Line(1, "Hell yeah!"), Line(2, "This is awesome"))
    block = Block()
    for line in lines:
        block += line
    assert tuple(block.lines) == lines


def test_is_empty():
    block = Block()
    assert block.is_empty()

    lines = (Line(1, "Hell yeah!"), Line(2, "This is awesome"))
    for line in lines:
        block += line
    assert not block.is_empty()
