
pragma page_size = 4096;
.load ./build/libmojo
.open 'file:a.db?vfs=mojo&pagesz=4096'

create table if not exists test (
    n int 
);

insert into test values (1);
insert into test values (2);
insert into test values (3);