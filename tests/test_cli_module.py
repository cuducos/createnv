from pathlib import Path


from createnv.cli import RANDOM_CHARS, cli, main


def test_main_with_default_values(mocker):
    parser = mocker.patch("createnv.cli.Parser")
    generator = mocker.patch("createnv.cli.Generator")
    main()
    parser.assert_called_once_with(Path(".env.sample"), RANDOM_CHARS)
    generator.assert_called_once_with(Path(".env"), parser.return_value, False, False)
    generator.return_value.assert_called_once_with()


def test_main_with_custom_values(mocker):
    parser = mocker.patch("createnv.cli.Parser")
    generator = mocker.patch("createnv.cli.Generator")
    main("env", "sample", True, True, "ab")
    parser.assert_called_once_with(Path("sample"), "ab")
    generator.assert_called_once_with(Path("env"), parser.return_value, True, True)
    generator.return_value.assert_called_once_with()


def test_cli(mocker):
    run = mocker.patch("createnv.cli.run")
    cli()
    run.assert_called_once_with(main)
