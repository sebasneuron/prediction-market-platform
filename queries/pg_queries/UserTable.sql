-- truncate table polymarket.users cascade;

-- delete from polymarket.users where id not in ('3a308ea2-003c-41e0-adeb-3bd1f7e0e884'::uuid, 'f45d9e53-820c-425b-b9bc-56af30ea1351'::uuid);
SELECT id, balance, email, name from polymarket.users;
-- SELECT * FROM polymarket.users;

-- delete from polymarket.users where id='cd31934d-0019-41d1-9ccc-bafc6b9330de';

-- update polymarket.users set balance=100 where balance=10.0 returning *;

-- update polymarket.users 
-- 	set balance = 
-- 	case
-- 		when id = '24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid then 100::numeric
-- 		when id = '27db8053-2640-45bf-894b-dcd420eb4886'::uuid then 100::numeric
-- 	end
-- 	where id in ('24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid, '27db8053-2640-45bf-894b-dcd420eb4886'::uuid);
		

-- select id,balance from polymarket.users where id in ('24fa20ac-822f-49e9-9cb6-e25e940ad608'::uuid, '27db8053-2640-45bf-894b-dcd420eb4886'::uuid);

