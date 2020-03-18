clean:
	@rm -rf .coverage
	@rm -rf .mypy_cache
	@rm -rf .pytest_cache
	@rm -rf .ropeproject
	@rm -rf createnv.egg-info
	@rm -rf dist
	@rm -rf htmlcov
	@find . -iname "*.pyc" | xargs rm 
	@find . -iname "__pycache__" | xargs rm  -rf
