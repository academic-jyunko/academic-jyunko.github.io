import re

def main():
	with open('Fayek.bib', 'r') as f:
		lines = f.read()

	bibs = lines.split('@')[1:]

	for bib in bibs:

		# bib = re.sub(r'\n(?=[^{}]*})', '', bib)  # remove new lines
		# bib = re.sub(r' +(?=[^{}]*})', ' ', bib)  # remove multiple space
		bib = bib.replace(',\n', '<>').replace(', \n', '<>').replace(',  \n', '<>').replace('\n','').replace('<>', ',\n')
		bib = re.sub(' +', ' ', bib)
		bib = '@' + bib.replace(',}', ',\n}') + '\n'  # add new line after the last field
		bib = bib.replace('\n ', '\n  ')

		with open(bib[bib.find('{') + 1:bib.find(',')] + '.bib', 'w') as f:
			f.write(''.join(bib))


if __name__ == '__main__':
    main()