do
$do$
declare
     i int;
begin
for  i in 0..200
loop
    DELETE FROM restaurant_table_orders WHERE table_number = i;
    DELETE FROM restaurant_tables  WHERE table_number = i;
end loop;
end;
$do$;
