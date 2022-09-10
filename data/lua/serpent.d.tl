local record serpent
	_NAME: string
	_VERSION: string
	_COPYRIGHT: string
	_DESCRIPTION: string

	serialize: function(data: any, opts: table): string
	deserialize: function(data: any, opts: table): string

	load: function(a: any, opts: table): string
	dump: function(a: any, opts: table): string
	line: function(a: any, opts: table): string
	block: function(a: any, opts: table): string
end

return serpent
