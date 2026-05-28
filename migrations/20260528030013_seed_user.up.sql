-- Add up migration script here
INSERT INTO users (user_id, username, password_hash, role)
VALUES ('c35b81bf-fa54-43fa-9b57-b090c7e47a24', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$UnJVMGt3YVA$n/Pt0+mSUG7qIB86e79OVvPkRETt1DK1FpR5PrOi5dc', 'admin');
