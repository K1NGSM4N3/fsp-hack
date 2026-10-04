--! insert_company
INSERT INTO companies (id, owner_user_id, name) VALUES (:id, :owner_user_id, :name);

--! company_of_owner : (industry?, website?, city?, size?, contact_email?, contact_phone?, contact_telegram?)
SELECT id, name, description, industry, website, city, size, contact_email, contact_phone, contact_telegram, updated_at
FROM companies
WHERE owner_user_id = :owner_user_id;

--! update_company (industry?, website?, city?, size?, contact_email?, contact_phone?, contact_telegram?)
UPDATE companies
SET name = :name, description = :description, industry = :industry, website = :website, city = :city,
    size = :size, contact_email = :contact_email, contact_phone = :contact_phone,
    contact_telegram = :contact_telegram, updated_at = now()
WHERE owner_user_id = :owner_user_id;
