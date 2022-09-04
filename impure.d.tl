global import: function(path: string): any

global record impure
	version: function(): number, number, number

	log: function(text: string)
	warn: function(text: string)
	err: function(text: string)
	debug: function(text: string)
end
