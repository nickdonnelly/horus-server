-- This file should undo anything in `up.sql`
DROP RULE IF EXISTS update_with_pastes_insert ON horus_pastes;
DROP RULE IF EXISTS update_with_videos_insert ON horus_videos;
DROP RULE IF EXISTS update_with_images_insert ON horus_images;
DROP RULE IF EXISTS update_with_files_insert ON horus_files;

DROP RULE IF EXISTS update_with_pastes_delete ON horus_pastes;
DROP RULE IF EXISTS update_with_videos_delete ON horus_videos;
DROP RULE IF EXISTS update_with_images_delete ON horus_images;
DROP RULE IF EXISTS update_with_files_delete ON horus_files;
