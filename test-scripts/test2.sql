
pragma page_size = 4096;
.load ./build/libmojo
.open 'file:./testdbs/a_1.db?vfs=mojo&pagesz=4096&mode=ro&ver=2'

select count(*), max(id) from person;