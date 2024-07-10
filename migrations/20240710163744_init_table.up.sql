do
$do$
declare
     i int;
begin
for  i in 0..200
loop
     INSERT INTO restaurant_tables (table_number) VALUES (i);
end loop;
end;
$do$;
