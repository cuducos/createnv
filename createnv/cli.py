from pathlib import Path
from string import ascii_letters, digits

from typer import run

from createnv.generator import Generator
from createnv.parser import Parser


RANDOM_CHARS = f"{ascii_letters}{digits}!@#$%^&*(-_=+)"


def main(
    target: str = ".env",
    source: str = ".env.sample",
    overwrite: bool = False,
    use_default: bool = False,
    chars_for_random_string: str = RANDOM_CHARS,
):
    """Creates a .env file with the environment variables following a sample
    .env.sample file. These defaults and other options can be changed. Check
    them with the --help option."""
    Generator(
        Path(target),
        Parser(Path(source), chars_for_random_string),
        overwrite,
        use_default,
    )()


def cli():
    run(main)
