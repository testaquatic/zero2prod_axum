-- 전체 마이그레이셔늘 트랜잭션으로 감싸서 단일하게 성공 또는 실패가 되도록 한다.
BEGIN;
-- 과거 데이터에 대한 `status`를 채운다.
UPDATE subscriptions
SET status = 'confirmed'
WHERE status is NULL;
-- `status`를 필수 컴럼으로 설정한다.
ALTER TABLE subscriptions
ALTER COLUMN status
SET NOT NULL;
COMMIT;