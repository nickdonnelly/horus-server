-- Your SQL goes here
CREATE RULE update_with_pastes_insert AS ON INSERT TO horus_pastes
  DO UPDATE horus_licenses SET resource_count = resource_count + 1 WHERE owner = new.owner;

CREATE RULE update_with_videos_insert AS ON INSERT TO horus_videos
  DO UPDATE horus_licenses SET resource_count = resource_count + 1 WHERE owner = new.owner;

CREATE RULE update_with_images_insert AS ON INSERT TO horus_images
  DO UPDATE horus_licenses SET resource_count = resource_count + 1 WHERE owner = new.owner;

CREATE RULE update_with_files_insert AS ON INSERT TO horus_files
  DO UPDATE horus_licenses SET resource_count = resource_count + 1 WHERE owner = new.owner;


CREATE RULE update_with_pastes_delete AS ON DELETE TO horus_pastes
  DO UPDATE horus_licenses SET resource_count = resource_count - 1 WHERE owner = old.owner;

CREATE RULE update_with_videos_delete AS ON DELETE TO horus_videos
  DO UPDATE horus_licenses SET resource_count = resource_count - 1 WHERE owner = old.owner;

CREATE RULE update_with_images_delete AS ON DELETE TO horus_images
  DO UPDATE horus_licenses SET resource_count = resource_count - 1 WHERE owner = old.owner;

CREATE RULE update_with_files_delete AS ON DELETE TO horus_files
  DO UPDATE horus_licenses SET resource_count = resource_count - 1 WHERE owner = old.owner;
