from functools import partial

import typer


echo = typer.echo
error = partial(typer.secho, fg="red")
success = partial(typer.secho, fg="green")
warning = partial(typer.secho, fg="yellow")
