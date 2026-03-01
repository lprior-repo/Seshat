.dbconfig defensive off
BEGIN;
PRAGMA writable_schema = on;
PRAGMA foreign_keys = off;
PRAGMA encoding = 'UTF-8';
PRAGMA page_size = '4096';
PRAGMA auto_vacuum = '0';
PRAGMA user_version = '1';
PRAGMA application_id = '0';
CREATE TABLE "dependencies" ("issue_id" TEXT, "depends_on_id" TEXT, "type" TEXT, "created_at" NUMERIC, "created_by" TEXT, "metadata" TEXT, "thread_id" TEXT);
CREATE TABLE "labels" ("issue_id" TEXT, "label" TEXT);
CREATE TABLE "comments" ("id" INTEGER PRIMARY KEY, "issue_id" TEXT, "author" TEXT, "text" TEXT, "created_at" NUMERIC);
CREATE TABLE "events" ("id" INTEGER PRIMARY KEY, "issue_id" TEXT, "event_type" TEXT, "actor" TEXT, "old_value" TEXT, "new_value" TEXT, "comment" TEXT, "created_at" NUMERIC);
CREATE TABLE "config" ("key" TEXT, "value" TEXT);
CREATE TABLE "metadata" ("key" TEXT, "value" TEXT);
CREATE TABLE "dirty_issues" ("issue_id" TEXT, "marked_at" NUMERIC);
CREATE TABLE "export_hashes" ("issue_id" TEXT, "content_hash" TEXT, "exported_at" NUMERIC);
CREATE TABLE "child_counters" ("parent_id" TEXT, "last_child" INTEGER);
CREATE TABLE "blocked_issues_cache" ("issue_id" TEXT, "blocked_by" TEXT, "blocked_at" NUMERIC);
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (54, 'bd-39r', 'bd-3sm', 'blocks', '2026-02-27T06:58:30.574807507+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (55, 'bd-3if', 'bd-2x8', 'blocks', '2026-02-27T07:03:58.129406430+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (56, 'bd-3qj', 'bd-2cg', 'blocks', '2026-02-27T07:04:00.328892722+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (57, 'bd-3sm', 'bd-2xa', 'blocks', '2026-02-27T06:58:29.670646079+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (58, 'bd-3t0', 'bd-2cg', 'blocks', '2026-02-27T07:03:59.303549905+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (59, 'bd-7rt', 'bd-2x8', 'blocks', '2026-02-27T07:04:00.201513559+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (60, 'bd-ahf', 'bd-1nb', 'blocks', '2026-02-27T07:04:00.656323228+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (61, 'bd-ahf', 'bd-2qg', 'blocks', '2026-02-27T07:04:00.721757213+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (62, 'bd-mtu', 'bd-7rt', 'blocks', '2026-02-27T07:04:00.266075342+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (63, 'bd-n05', 'bd-2cg', 'blocks', '2026-02-27T07:03:59.492654508+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (64, 'bd-n05', 'bd-vh4', 'blocks', '2026-02-27T07:03:59.556750565+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (65, 'bd-s62', 'bd-1nb', 'blocks', '2026-02-27T07:03:59.104245576+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (66, 'bd-vh4', 'bd-3if', 'blocks', '2026-02-27T07:03:58.192606816+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (67, 'bd-x3x', 'bd-2cg', 'blocks', '2026-02-27T07:03:59.944656212+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (68, 'bd-x3x', 'bd-34b', 'blocks', '2026-02-27T07:03:59.877101655+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (1, 'bd-104', 'bd-3ve', 'blocks', '2026-02-27T06:58:29.862316566+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (2, 'bd-19h', 'bd-1nb', 'blocks', '2026-02-27T07:04:00.009856840+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (3, 'bd-19h', 'bd-1zz', 'blocks', '2026-02-27T07:04:00.073105165+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (4, 'bd-19h', 'bd-2kq', 'blocks', '2026-02-27T07:04:00.136926776+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (5, 'bd-19t', 'bd-s62', 'blocks', '2026-02-27T07:04:00.849246725+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (6, 'bd-19t', 'bd-x3x', 'blocks', '2026-02-27T07:04:00.914856270+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (7, 'bd-1et', 'bd-n05', 'blocks', '2026-02-27T07:03:59.621827214+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (8, 'bd-1gl', 'bd-1nb', 'blocks', '2026-02-27T07:03:59.039508513+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (9, 'bd-1ik', 'bd-2mk', 'blocks', '2026-02-27T06:58:29.764232468+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (10, 'bd-1kc', 'bd-2cg', 'blocks', '2026-02-27T07:03:59.366313525+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (11, 'bd-1lj', 'bd-2cg', 'blocks', '2026-02-27T07:03:59.428988896+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (12, 'bd-1nb', 'bd-1rv', 'blocks', '2026-02-27T07:03:58.652211841+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (13, 'bd-1nb', 'bd-1zz', 'blocks', '2026-02-27T07:03:58.720026765+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (14, 'bd-1nb', 'bd-2hz', 'blocks', '2026-02-27T07:03:58.784320541+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (15, 'bd-1nb', 'bd-2o1', 'blocks', '2026-02-27T07:03:58.846794384+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (16, 'bd-1rv', 'bd-vh4', 'blocks', '2026-02-27T07:03:58.257736824+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (17, 'bd-1ua', 'bd-1nb', 'blocks', '2026-02-27T07:03:58.910880582+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (18, 'bd-1wc', 'bd-2cg', 'blocks', '2026-02-27T07:04:00.591042631+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (19, 'bd-1ws', 'bd-1gl', 'blocks', '2026-02-27T07:04:00.460911723+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (20, 'bd-1ws', 'bd-1zz', 'blocks', '2026-02-27T07:04:00.526537607+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (21, 'bd-1ws', 'bd-3qj', 'blocks', '2026-02-27T07:04:00.394487266+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (22, 'bd-1zz', 'bd-n75', 'blocks', '2026-02-27T07:03:58.321887652+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (23, 'bd-20k', 'bd-2mk', 'blocks', '2026-02-27T06:58:29.145344306+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (24, 'bd-2b6', 'bd-2mk', 'blocks', '2026-02-27T06:58:29.238250021+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (25, 'bd-2cg', 'bd-1nb', 'blocks', '2026-02-27T07:03:59.238986452+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (26, 'bd-2cg', 'bd-2hz', 'blocks', '2026-02-27T07:03:59.169059427+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (27, 'bd-2hz', 'bd-1zz', 'blocks', '2026-02-27T07:03:58.388467127+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (28, 'bd-2ik', 'bd-2xa', 'blocks', '2026-02-27T06:58:30.063131320+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (29, 'bd-2ik', 'bd-33b', 'blocks', '2026-02-27T06:58:29.964164800+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (30, 'bd-2kq', 'bd-2hz', 'blocks', '2026-02-27T07:03:58.517123078+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (31, 'bd-2kq', 'bd-2o1', 'blocks', '2026-02-27T07:03:58.585772595+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (32, 'bd-2mk', 'bd-3lk', 'blocks', '2026-02-27T06:58:29.053658880+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (33, 'bd-2mk', 'bd-3ve', 'blocks', '2026-02-27T06:58:28.949895603+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (34, 'bd-2o1', 'bd-1zz', 'blocks', '2026-02-27T07:03:58.452297577+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (35, 'bd-2qg', 'bd-1ua', 'blocks', '2026-02-27T07:03:58.975438806+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (36, 'bd-2x8', 'bd-189', 'blocks', '2026-02-27T07:03:58.000659190+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (37, 'bd-2x8', 'bd-26j', 'blocks', '2026-02-27T07:03:57.863256078+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (38, 'bd-2x8', 'bd-2v0', 'blocks', '2026-02-27T07:03:57.795394124+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (39, 'bd-2x8', 'bd-3ex', 'blocks', '2026-02-27T07:03:57.931887624+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (40, 'bd-2x8', 'bd-n75', 'blocks', '2026-02-27T07:03:58.065911127+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (41, 'bd-2xa', 'bd-33b', 'blocks', '2026-02-27T06:58:29.540210495+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (42, 'bd-320', 'bd-n05', 'blocks', '2026-02-27T07:04:00.785532224+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (43, 'bd-33b', 'bd-2mk', 'blocks', '2026-02-27T06:58:29.439187784+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (44, 'bd-33b', 'bd-3lk', 'blocks', '2026-02-27T06:58:29.339585890+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (45, 'bd-34b', 'bd-1rv', 'blocks', '2026-02-27T07:03:59.813642212+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (46, 'bd-34b', 'bd-38a', 'blocks', '2026-02-27T07:03:59.750189448+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (47, 'bd-38a', 'bd-2x8', 'blocks', '2026-02-27T07:03:59.685560875+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (48, 'bd-39r', 'bd-104', 'blocks', '2026-02-27T06:58:30.696647769+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (49, 'bd-39r', 'bd-20k', 'blocks', '2026-02-27T06:58:30.270470784+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (50, 'bd-39r', 'bd-2ik', 'blocks', '2026-02-27T06:58:30.791414887+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (51, 'bd-39r', 'bd-2mk', 'blocks', '2026-02-27T06:58:30.162923112+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (52, 'bd-39r', 'bd-2xa', 'blocks', '2026-02-27T06:58:30.469713452+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'dependencies'(_rowid_, 'issue_id', 'depends_on_id', 'type', 'created_at', 'created_by', 'metadata', 'thread_id') VALUES (53, 'bd-39r', 'bd-33b', 'blocks', '2026-02-27T06:58:30.368184246+00:00', 'lewis', '{}', '');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (1, 'bd-104', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (2, 'bd-1ik', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (3, 'bd-20k', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (4, 'bd-2b6', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (5, 'bd-2ik', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (6, 'bd-2mk', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (7, 'bd-2xa', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (8, 'bd-33b', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (9, 'bd-39r', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (10, 'bd-3lk', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (11, 'bd-3sm', 'superseded');
INSERT OR IGNORE INTO 'labels'(_rowid_, 'issue_id', 'label') VALUES (12, 'bd-3ve', 'superseded');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (1, 'bd-104', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T16:02:14.048045837+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (2, 'bd-104', 'assignee_changed', 'lewis', NULL, 'self', NULL, '2026-03-01T16:02:14.048047777+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (3, 'bd-3lk', 'status_changed', 'lewis', 'in_progress', 'closed', NULL, '2026-03-01T16:02:20.861784908+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (4, 'bd-3lk', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T16:02:20.861785468+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (5, 'bd-2mk', 'status_changed', 'lewis', 'open', 'closed', NULL, '2026-03-01T16:02:20.893190089+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (6, 'bd-2mk', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T16:02:20.893191059+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (7, 'bd-20k', 'status_changed', 'lewis', 'open', 'closed', NULL, '2026-03-01T16:02:20.925080757+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (8, 'bd-20k', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T16:02:20.925082007+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (9, 'bd-2b6', 'status_changed', 'lewis', 'open', 'closed', NULL, '2026-03-01T16:02:20.957690749+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (10, 'bd-2b6', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T16:02:20.957692629+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (11, 'bd-104', 'status_changed', 'lewis', 'in_progress', 'closed', NULL, '2026-03-01T16:12:54.789854535+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (12, 'bd-104', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T16:12:54.789855605+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (13, 'bd-1ik', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T16:13:18.771660625+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (14, 'bd-1ik', 'assignee_changed', 'lewis', NULL, 'self', NULL, '2026-03-01T16:13:18.771662785+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (15, 'bd-1ik', 'status_changed', 'lewis', 'in_progress', 'closed', NULL, '2026-03-01T16:15:33.442883731+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (16, 'bd-1ik', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T16:15:33.442884421+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (17, 'bd-33b', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T18:52:23.899590332+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (18, 'bd-33b', 'assignee_changed', 'lewis', NULL, 'self', NULL, '2026-03-01T18:52:23.899592602+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (19, 'bd-19t', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T18:58:27.159643043+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (20, 'bd-19t', 'assignee_changed', 'lewis', NULL, 'lewis', NULL, '2026-03-01T18:58:27.159644953+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (21, 'bd-19h', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T19:01:12.544866203+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (22, 'bd-19h', 'assignee_changed', 'lewis', NULL, 'lewis', NULL, '2026-03-01T19:01:12.544867863+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (23, 'bd-33b', 'status_changed', 'lewis', 'in_progress', 'closed', NULL, '2026-03-01T19:02:29.587764658+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (24, 'bd-33b', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T19:02:29.587765348+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (25, 'bd-2xa', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T19:02:43.644713366+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (26, 'bd-2xa', 'assignee_changed', 'lewis', NULL, 'self', NULL, '2026-03-01T19:02:43.644715126+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (27, 'bd-q2i', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T19:03:45.690800083+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (28, 'bd-1et', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T19:10:38.947845121+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (29, 'bd-1et', 'assignee_changed', 'lewis', NULL, 'lewis', NULL, '2026-03-01T19:10:38.947847051+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (30, 'bd-1td', 'status_changed', 'lewis', 'open', 'closed', NULL, '2026-03-01T19:10:40.686494342+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (31, 'bd-1td', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T19:10:40.686496502+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (32, 'bd-yf9', 'status_changed', 'lewis', 'open', 'closed', NULL, '2026-03-01T19:12:33.499249994+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (33, 'bd-yf9', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T19:12:33.499251034+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (34, 'bd-10k', 'status_changed', 'lewis', 'open', 'closed', NULL, '2026-03-01T19:13:08.105517032+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (35, 'bd-10k', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T19:13:08.105517692+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (36, 'bd-2xa', 'status_changed', 'lewis', 'in_progress', 'closed', NULL, '2026-03-01T19:17:01.440420473+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (37, 'bd-2xa', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T19:17:01.440420953+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (38, 'bd-3sm', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T19:18:33.258871175+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (39, 'bd-3sm', 'assignee_changed', 'lewis', NULL, 'self', NULL, '2026-03-01T19:18:33.258873035+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (40, 'bd-3sm', 'status_changed', 'lewis', 'in_progress', 'closed', NULL, '2026-03-01T19:22:13.891517794+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (41, 'bd-3sm', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T19:22:13.891518584+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (42, 'bd-2ik', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T19:23:08.967652739+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (43, 'bd-2ik', 'assignee_changed', 'lewis', NULL, 'lewis', NULL, '2026-03-01T19:23:08.967654399+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (44, 'bd-2ik', 'status_changed', 'lewis', 'in_progress', 'in_progress', NULL, '2026-03-01T19:23:55.282033002+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (45, 'bd-2ik', 'assignee_changed', 'lewis', 'lewis', 'self', NULL, '2026-03-01T19:23:55.282034982+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (46, 'bd-q2i', 'status_changed', 'lewis', 'in_progress', 'closed', NULL, '2026-03-01T19:28:00.681831583+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (47, 'bd-q2i', 'closed', 'lewis', NULL, NULL, 'done', '2026-03-01T19:28:00.681832363+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (48, 'bd-n05', 'status_changed', 'lewis', 'open', 'in_progress', NULL, '2026-03-01T19:29:49.915531422+00:00');
INSERT OR IGNORE INTO 'events'('id', 'issue_id', 'event_type', 'actor', 'old_value', 'new_value', 'comment', 'created_at') VALUES (49, 'bd-n05', 'assignee_changed', 'lewis', NULL, 'lewis', NULL, '2026-03-01T19:29:49.915533092+00:00');
INSERT OR IGNORE INTO 'config'(_rowid_, 'key', 'value') VALUES (1, 'issue_prefix', 'bd');
INSERT OR IGNORE INTO 'metadata'(_rowid_, 'key', 'value') VALUES (50, 'last_export_time', '2026-03-01T19:28:01.372264645+00:00');
INSERT OR IGNORE INTO 'metadata'(_rowid_, 'key', 'value') VALUES (51, 'last_import_time', '2026-03-01T19:28:36.109147601+00:00');
INSERT OR IGNORE INTO 'metadata'(_rowid_, 'key', 'value') VALUES (52, 'jsonl_content_hash', '13e1d4f9cd33235930ac451154900a6a1c9e635a3d7d4d000addb9f655dc7ac8');
INSERT OR IGNORE INTO 'dirty_issues'(_rowid_, 'issue_id', 'marked_at') VALUES (1, 'bd-n05', '2026-03-01T19:29:49.916460863+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (138, 'bd-yf9', '19b274261c62bd5f4a74a224790a46b639848d67608a5b88454dfe84938204a0', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (1, 'bd-104', '9b8e6ed19d88daf8fef00a07169c7aa23500c873dd9f882130ee7a3737b4035c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (2, 'bd-10k', '63b23faff9c9e342aeae314dbb8c73bab592a4dede604f41e889f32deec96f85', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (3, 'bd-11b', '628b159300633c5377f63dc11f2b89fc99733502be97da4d056686dac864aa5e', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (4, 'bd-11c', '42349572692a12009dba8ef127b1951349e64f7b1f99a5f74994393212c6f12c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (5, 'bd-12b', '8585f7a84d2ee78ca82d03ac34c1c3a346db72baad1d6a39a3cc84349c4c3fec', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (6, 'bd-163', '207eb0bb10770540eb9481d7f8e8a7d8db4e8a5331c87cc1009d647f29b98771', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (7, 'bd-17y', 'c12cc5283d9ed912b07d0e04e93f131676dd882a422aab5f2d58d6fece3d91f6', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (8, 'bd-189', 'c99a930ac23b38aacd85af4a734d715800417975e580f07b8a190985cb953f4c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (9, 'bd-19h', 'b2d5ffe24038d0e4b8cf0bb6847b259648a8e095a4b53c49170a11a1af980b0d', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (10, 'bd-19t', '384ff04b58e24803323f69eba8c423e0f95bcdf6a97ec6d134e52911af226b21', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (11, 'bd-1ag', 'e9212d655c4c50f373570cc002d801d7ec50e02537f3f3131f3fbbe2f583984e', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (12, 'bd-1db', 'b05838cceeb77572a4dc0d7f32a44c2e1a29a21c16461d66508db8bc943f3499', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (13, 'bd-1et', '8b92d0a7943ba0e737b5f98f2a79495348e09dd02399be7e7461002598e2fa23', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (14, 'bd-1fc', '485e5a4fc44e14334169b5505b3d113809a149b1a011b253746a17ff0bf3e908', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (15, 'bd-1gl', 'ac932eb6ba0cbb03e884771c168426c745a4fce3d1f56c4da3d1324840f68d36', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (16, 'bd-1ik', 'd57ec29174d6e602e15cfb720171573fe2db77eb4062feb66d61d408556bf158', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (17, 'bd-1jm', 'e5878a1ebd011311a031afe381cbd53761d3d7ec2eff49b51c83e1f97840ca47', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (18, 'bd-1kc', 'ca4f1f22b2512cc173479942ff953f469b186d0eee8c2a1dccd41b65706acc9c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (19, 'bd-1lj', 'bfe1a199024c84aa10d8951dba914911e811465a4bba70b491d9423a83bcea9d', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (20, 'bd-1nb', '9741012571863a48c98616d14e6fdf6d32ebdf0af97bb4efe1cff655a27d496a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (21, 'bd-1qt', 'e99b9edc3d383d1b35c35c7058d568c5b2d17cecd50b82df9514ce8ff2a5672f', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (22, 'bd-1rv', '8c88dbcac3d9fbbe0125337089a4c6e580594508e42915dcabd3ac06065e4040', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (23, 'bd-1s3', '7bd64088e881b9cef3107c384dd977b2276396721f988bd433efb8cb8aab1682', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (24, 'bd-1td', 'c5c10cefdc7060035e9bffd56840adef8c0f92ac0f8370cc6c89e63691911f47', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (25, 'bd-1u1', 'e33792413459ade9420464e21c80d08e1a5e26b98b8210b7a2f3d6f2e4e48979', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (26, 'bd-1u4', '350944f90e9b7ce039017faf5f27fc4f76e761d9a9c9c6e4f1c42e0a1c54b258', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (27, 'bd-1ua', '23de4461b4c1b284658b6cf4792c236d63771e62117b471d01644382702f49f0', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (28, 'bd-1ub', '8b1e3849088a477181488335abdc99368d9d013428f18c0c2cc4c51deb421539', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (29, 'bd-1wc', '83c99d24fe9c70652d85c130ecebde5a3c4ee3263d06b831308e92488e6424b6', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (30, 'bd-1ws', 'ac1cb0c3fee9a7d2e1c88c8983ccb4c0bf01adffa5937685d8bcd2ed4778dbe5', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (31, 'bd-1wz', '743871816da089992eb087a3540f36d2a3d1a9619e9c2d69865fac127871c9f6', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (32, 'bd-1x4', 'f7dac89972db46dd0872850793556a5f9be4788881f535393ac626d0febb8d6c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (33, 'bd-1yu', 'ddeb8f1e57c75c3e086fca172e33e23c114a3b61fd33fe87ace3f09da00b91a7', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (34, 'bd-1zz', '4e9582142ef8206174e639c554f985b8bd660568c4b3009177760267613b373c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (35, 'bd-20k', '72f77028747a275f16e3c51afd7d0760c097164ff5208917bd1e655a94f5e3f8', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (36, 'bd-21j', '5e2cb53d2916c55fdc48dd6b180d4993162f539719e1076803305a0c7efade6f', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (37, 'bd-23z', 'c085111c7e926b552479f872f7cbf7e8fb7e8ed14c3cc60de6d4c1ec08ab4bd4', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (38, 'bd-24a', 'c0216f2acd8e54eac2374604e396266268ab9e92657e76b01cb0eae2a3cd0346', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (39, 'bd-25b', 'b9b5332bbd2dad69426a4b0e90ef00cf0de29f452450b00aa6dbb2ecd2887807', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (40, 'bd-260', '0a78706e3e8a434f4da9a2839028eb7598a1abf7de34e285b606e1b63893f99f', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (41, 'bd-26j', 'c8e8fd782a0c915c5d1003dd00079268d1da3ab198ba97bfb9365dcf6498258e', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (42, 'bd-275', '6d69f1968eae2f46350ebbf6112e93004caa872f2be7eb1fae5329e1f624de5a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (43, 'bd-27q', '4ba815c310745c6e8609e76b378e371bb5f6418e49b37fd274d2bcd79ac43e15', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (44, 'bd-29g', '295ea1d9d7748918cecd2fbf2b3551b8252cf8b7c949c262f94f34f05e75ce77', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (45, 'bd-2b4', '0cd985a5c3e8d1a855eb91156d7b503aeaef847c5535d050945cb4e952e40386', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (46, 'bd-2b6', 'ed9c5b67e467d1b978d925928d320284c9a3eb0a4bf8509fdb78399ccd66b779', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (47, 'bd-2bh', '0d94138f76efccb4b2e23925e4cddd69bac853c648353163b752438ffc53532b', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (48, 'bd-2cb', '3406639168238754da1a9a766f48b35c7653ee7beafe61b52e55a307aaa586c6', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (49, 'bd-2cg', '0ffae1219bf876d95f991047b87b2ef1b4bad86964142f2571ecc127efdd0074', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (50, 'bd-2cm', '4ead30fa13162734972df934c2012e336a9c13ae7399f2d52e74cace40bd0afd', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (51, 'bd-2df', 'fdf8299b0ed621c9c00e43c4d5aeb704088b98257a7eb3f38882752ca06a567d', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (52, 'bd-2dg', 'beed3b123a4441215970b7708cf42226c1620da78517217891809df606d44c17', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (53, 'bd-2gl', '8016b819a0f1e8ab55b2f931091275f1acc1975fb07f009a206edc9ede40bf6f', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (54, 'bd-2hz', 'ad9e59d8844aa2086a5bddfbe95a087f62005c336dc6c8b6b7d305eb9c76171c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (55, 'bd-2ii', '72c4a5ee7b47ec63873234221dd3a21ec4550e1000d04177b2c4cf7c97e7213c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (56, 'bd-2ik', '9fd68f369cd2b83b2a4e19f092495706006fafac561e61f98a4abe4334f9eecd', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (57, 'bd-2jh', 'afafe4ac9c6d3b84bd4af572ce0f1afb01d4a1a8e9a85de233d6b891d9cbf0a0', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (58, 'bd-2jv', '83d40916ee7c712da821ffdb02688527d387a235a993bce0a659b0ed44bd00d5', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (59, 'bd-2kq', 'f800b7f9234207a14c1527eed0335cfaeb8067e04dfa2dfc81110452a2c0503a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (60, 'bd-2kx', '925200b8025ce0bf7e4dc7d3d19ef702a8fef20cdc97d8c1a4cc639dff83a179', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (61, 'bd-2l6', 'fc16444c3463ee7d58069f79bf5e33e0d0223e79bba33d33b3dcd8c8d4228a7b', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (62, 'bd-2mk', '2d178c9b2b3b38cf89d5cf6d6d3e8e7378607f8907478ec16f657e9a08f7eee3', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (63, 'bd-2ng', 'd02177797689f28b08b816917b2adbcc09ac79ef206d37ef5ed34a6cba040e15', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (64, 'bd-2o1', '98b2e22428544fd18aa2e0b4dc1cee6e99403dd8af23d72ada42c07e044a3fe9', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (65, 'bd-2p8', '0560e35fa81d9a509dee4d67c5680665090bbcc62f3b2108a6fa5758089d13e9', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (66, 'bd-2qg', '1fc057db9d993d36cfd6f223cfb6a8d0cfb6699bb29964204e9047948619a39c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (67, 'bd-2sa', '471778134cb28c79f28c0c9988e23b7acbe58dd96ca20b6ac7a872b9e436f358', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (68, 'bd-2tx', 'e8d8156eb07d89a80af9e87a27a607e27483f3b30b36ff21cc5e4cad14411010', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (69, 'bd-2u3', '44342e53fbf735f0978a40686107bb35cbffaf1916ca6a80f77462ef72a68a82', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (70, 'bd-2uy', 'b8667fd72f62a2ec28dcb4bee8cb709a85bdf879f590c2ff9c9f7a52e6fd971a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (71, 'bd-2v0', '8aaf4a1067eee4ae6d3f41619584dfa04c51ebfa97cf981092461a9711e6077e', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (72, 'bd-2vf', '4c0a4288e805e4ef8c1dd8f439b39c0fc3f74c7d86aa3ee084937912c1b2f87b', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (73, 'bd-2w1', '4ef2eae4aeb7665317cc0178583d53004299591708fd8a3d349d7b465d6effd5', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (74, 'bd-2x8', '64cb41944c6ae9fcaf3f600596be0f7c201f0381c4d15c60ad540dc4b2dd6e29', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (75, 'bd-2xa', 'bda57fdad7f99cb96edf8548d9a58b35ddb8ce4698a439c6708ecae7e62bafa7', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (76, 'bd-2xt', '4320af136f6ad5b9d9a6401c37d20eebb12e3c2bd04f238c12fb839838f5e38d', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (77, 'bd-30o', '2674c2fb16c9f420d56b2b54d8a35e6802b38d20169286cb041538aed180ad5c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (78, 'bd-318', '47ad1fbabfb6dda55e5e8938ab746107ccb69eeb102d27cb93b116ed528cb512', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (79, 'bd-31x', '3f764ec12e58b58ba5b9d02b44681ae3f3914ad0927b92be3510208abe94fe9c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (80, 'bd-320', '2683188c4e02f0d36c11f75945f91740ac5d15761799b69ce18af081344fa3e1', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (81, 'bd-321', 'fd9cae6d2203605006e6e8ae4086f62ebcfe0b8eba64885aa7a554dbadcdbe54', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (82, 'bd-338', '4c0cb745450164e091505c1e241bd57c472718ac1bc1711f2166269e418b5015', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (83, 'bd-33b', '6004595a764ae3d01b565fc0f5e649719944faad670cf6e28507d514feb89aaa', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (84, 'bd-345', 'b3cfbc678ccf5d6e3db33e6850362bd2bb2acf8160c962ba986062081c13cf73', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (85, 'bd-34b', 'bd13ed2aeccfb7f934edcac641bd7646e923bd9d8fc1825c2f0ec14e5516395a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (86, 'bd-34q', '3046c0fadd2c497bb2cc08190e540dc9237a377cc15c8cdee8d4cec4500aae7c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (87, 'bd-354', 'dcab80773127a2b79ca75f799f53721d3688e220f33352b40a35f9dc21ac9853', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (88, 'bd-37e', '4f8d982942557343b3ee943a68a21848d07fc4ffd09178c41c5f0897cc1dc20d', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (89, 'bd-37x', 'eea09f0d9955518e07bc6b5ca01388e070853c80e086db10d923e82c909dde1f', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (90, 'bd-387', '96607dde03645139a8127e2021286b4664ffe3ff49de8a4955f639308c30da65', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (91, 'bd-38a', '94177d8c469fe0b0bf9fa41aff1bee5d9d42cef072f7528ec314bcdfa4db0b5a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (92, 'bd-39r', '83cf52bef8805dc138527e8bc20c4f6fcf0bdc6c2cd56cda96eb725bdb34ad51', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (93, 'bd-3dc', '13dfbeeaa45dd18460ab0d5f7f68e63de7e47a2424da1a417cd935ffaf82d915', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (94, 'bd-3ex', '5fb7a1b7d5dd5dc7617a0c1133060fc78720daacba7bd46bbf8948dd7f80ca97', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (95, 'bd-3ic', 'b4ed9bc0d9f1e017c31a2eca652c25d3499eac751b79ddcdad5a505980869fe7', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (96, 'bd-3if', '9fa79b28fc6d29510cdf5ca1eade29fba39fc42aed9719f51a6ca9d24de81b8b', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (97, 'bd-3lk', '5d0c4639e43e2e4514febeb0929f581f27fbdddff0a72e869654bd5a47672fdd', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (98, 'bd-3pt', '490cd2c821626a92dd130d7e483b6b5725b91043e8e4de474eea54033b7ad686', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (99, 'bd-3qj', 'fcb9d392287dac3d231b554bef6d417858791a7b1df7f42ccb0e8d2d2655e728', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (100, 'bd-3rb', '6f932af181d8c0c8747f0b569bb053c6819e5cfbaa1602afed60684c4d099a57', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (101, 'bd-3sm', '969aea17d94630ffea357be03c6431c59142378b30112c2311085b098cbf1362', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (102, 'bd-3t0', '7c37ff58745763b27717e5aad89035ab03a1e4f0106d5f0353667418b9d48a10', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (103, 'bd-3uq', '578916dd8bacdee221ec5e72fe74f064b74bd978b1b7e1e949a15b9c0ab02c9b', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (104, 'bd-3ve', 'e0e8827440427f2df46e7246e157ad0c32b8e2b69110f1d234b4d59a449bc437', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (105, 'bd-3vt', '83aa935b1dde19a39e0b5bf342fea1b8d32790b5335dc5f52435f3d58c7900f8', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (106, 'bd-51b', 'f642244fddf482657fa4d279853ce494a00f9cf6d8a55a2fe5559a33f05ced56', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (107, 'bd-5id', '3b91dbdbdf182c3a9b57153b1f3057656e0c15c43030df2199db8d4e5e562753', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (108, 'bd-5jh', '0c8e0e1c9a6b2aa3d4cfc1b4c6edc25bcac1d8741afbedde515b202cbbdbc12b', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (109, 'bd-7rt', 'f4d24f80475214ff6e0e62fa06b87c59219d60dc58104e17c5bbdcd6289c3f40', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (110, 'bd-81l', 'a19190b7e817792a5fb84ce2b0a9e8b6f8a85179a524b09721a3852cd0d4d326', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (111, 'bd-8sr', '169236d7bba786a2b16de2facd04506e1db107cab963c73ab8d6ef56f2572efc', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (112, 'bd-a11', 'd8e6c324b29a4029722ecbbc8f4bdc3db53f759987eacb3ed87b6de5a26da70b', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (113, 'bd-ahf', 'fdb2d8a98822377ee4ce2c807ae13a302cd195e09500b7f74b1e4a934d979b5f', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (114, 'bd-due', 'dc54a28dda78665d50bcfffa29372ff6cdb727a69c6c2bfc3498c42299854e88', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (115, 'bd-ja2', 'c97980bf04def34d077876d756a160a988d715606efa079590cbfca34df52e16', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (116, 'bd-jqu', '59014799d9bb5cead30e64c4af5c3a8594af4c70149ef1dff7b07a38c5a7a3b9', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (117, 'bd-l79', '4d93e62498c062366e49124ec67d268b56e0afa07a6e754626b4101a061abd1a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (118, 'bd-lgh', 'ae1f5fdce1b33daa81f9d7f1022bb5f8f6016c1cad193138bbcc8e585245108f', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (119, 'bd-lq3', 'e785dfa01e4c64b2330694ee937af9e8e37e5f4f7b55fc15ae2bdcb0bd5cf6cd', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (120, 'bd-mtu', 'd8b750ac4429327e2863d62090512c2389569b4ccf49a78607c502ca985a9211', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (121, 'bd-n05', 'e23ba26fc93830a48c2ac311747d9d7e9cd462f562d4f198376490eaacbbb346', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (122, 'bd-n75', '1f29e59b46e24d1ad1a48804daa8a17fa83945f68e6fe8c61dae20254480b5e6', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (123, 'bd-nbm', '01cdb7889086873a44155a7adc0b5c82dbcdf0e0c505e4ae9f70b0267cf9833c', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (124, 'bd-o6p', '7ae8ebed9507fc97b9c4ade53be4eb89bf6a9076ca04d4d5c1127a217cbaa89a', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (125, 'bd-ohy', '1704cdc80b5ae63b26d11ae1d73388bfcfde4630bc6f83357d677abe3bb85680', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (126, 'bd-pmf', '97d52b73a3f70bc8e7089328a15706f2ab573469df7a29ece3af9bfdf2065df0', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (127, 'bd-q2i', 'ab3cbdb193fde32501a7e5c858b24e579a41e7dc0c06c288d1df5b15de930f81', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (128, 'bd-rg7', '23f5c180367975e0354e7475a5021bcdbb8fe1aef97c7452f1b3e474c741a506', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (129, 'bd-rn3', '08e27ca42287e478acb20cf6ebf41b1e793d792669d17fb6f45ca69a3c7b81f7', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (130, 'bd-rpv', '692c336504b60f569183cc544aa4a32aa930bf3079f21f1566011bfaf2390e60', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (131, 'bd-s62', 'fb020749d77aaf5f65f67b59ebea967de5191aef8ff1e7cc9740efafe6b33a9e', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (132, 'bd-sa6', '53a40bf88ab731697a365763c6fac77b3a021a1c8c16665c6da3e7e4a2665ef7', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (133, 'bd-tet', 'c0983032a17d2b1a0ef19db33be4804bff56d2021edf641d99ac4925d98d2594', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (134, 'bd-uem', '7a2f1fb78d9e55d338a705924af41a0be74f72c8d1e14373df1acbb50788455d', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (135, 'bd-v3y', '59c47adf299d3838e022890c7cc660db3dd54f4ec7c0a28dd56eaa4c8bba1c45', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (136, 'bd-vh4', 'e8757cb86f427f0914e7292824cf066ce960be8d2ad13d1f0ca7bd3fa4c19886', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'export_hashes'(_rowid_, 'issue_id', 'content_hash', 'exported_at') VALUES (137, 'bd-x3x', '2e5bc44519412d78d3d3cff26d6bbd005dd7ef223c2f20b303f7b694bb58e836', '2026-03-01T19:28:35.566830173+00:00');
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (1, 'bd-n05', '["bd-2cg:open","bd-vh4:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (2, 'bd-1ua', '["bd-1nb:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (3, 'bd-1et', '["bd-n05:in_progress"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (4, 'bd-1ws', '["bd-1gl:open","bd-3qj:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (5, 'bd-1wc', '["bd-2cg:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (6, 'bd-2kq', '["bd-2hz:in_progress","bd-2o1:in_progress"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (7, 'bd-2qg', '["bd-1ua:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (8, 'bd-34b', '["bd-1rv:open","bd-38a:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (9, 'bd-3t0', '["bd-2cg:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (10, 'bd-320', '["bd-n05:in_progress"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (11, 'bd-2cg', '["bd-1nb:open","bd-2hz:in_progress"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (12, 'bd-1nb', '["bd-1rv:open","bd-2hz:in_progress","bd-2o1:in_progress"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (13, 'bd-3qj', '["bd-2cg:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (14, 'bd-1lj', '["bd-2cg:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (15, 'bd-1kc', '["bd-2cg:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (16, 'bd-1gl', '["bd-1nb:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (17, 'bd-39r', '["bd-2ik:02-27T07:03"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (18, 'bd-19t', '["bd-s62:open","bd-x3x:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (19, 'bd-x3x', '["bd-2cg:open","bd-34b:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (20, 'bd-19h', '["bd-1nb:open","bd-2kq:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (21, 'bd-s62', '["bd-1nb:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (22, 'bd-ahf', '["bd-1nb:open","bd-2qg:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (23, 'bd-mtu', '["bd-7rt:open"]', NULL);
INSERT OR IGNORE INTO 'blocked_issues_cache'(_rowid_, 'issue_id', 'blocked_by', 'blocked_at') VALUES (24, 'bd-1rv', '["bd-vh4:open"]', NULL);
CREATE TABLE lost_and_found(rootpgno INTEGER, pgno INTEGER, nfield INTEGER, id INTEGER, c0, c1, c2, c3, c4, c5, c6, c7, c8, c9, c10, c11, c12, c13, c14, c15, c16, c17, c18, c19, c20, c21, c22, c23, c24, c25, c26, c27, c28, c29, c30, c31, c32, c33, c34, c35, c36, c37);
INSERT INTO lost_and_found VALUES(164, 164, 38, 76, 'bd-2xt', '4320af136f6ad5b9d9a6401c37d20eebb12e3c2bd04f238c12fb839838f5e38d', 'grid: Visual dot grid overlay on canvas', '# CUE Validation Schema
# Validate implementation: cue vet /home/lewis/src/whiteboard_kit/.beads/schemas/Seshat-20260223103614-rh1ocyvg.cue implementation.cue
# Schema location: /home/lewis/src/whiteboard_kit/.beads/schemas/Seshat-20260223103614-rh1ocyvg.cue


#EnhancedBead: {
  id: "Seshat-20260223103614-rh1ocyvg"
  title: "grid: Visual dot grid overlay on canvas"
  type: "feature"
  priority: 1
  effort_estimate: "30min"
  labels: ["planner-generated"]

  clarifications: {
    clarification_status: "RESOLVED"
  }

  ears_requirements: {
    ubiquitous: [
      \"THE SYSTEM SHALL display a dot grid background on the canvas\",
      \"THE SYSTEM SHALL scale grid dots with viewport zoom\"
    ]
    event_driven: [
      {trigger: \"WHEN viewport zoom changes\", shall: \"THE SYSTEM SHALL update grid background size to maintain visual consistency\"}
    ]
    unwanted: [
      {condition: \"IF grid overlay affects performance negatively\", shall_not: \"THE SYSTEM SHALL NOT render grid with excessive DOM elements\", because: \"Performance must remain smooth during pan and zoom\"}
    ]
  }

  contracts: {
    preconditions: {
      auth_required: false
      required_inputs: []
      system_state: [
        \"Canvas container has defined dimensions\",
        \"Viewport has valid zoom and pan values\"
      ]
    }
    postconditions: {
      state_changes: [
        \"Grid dots are evenly spaced visually\",
        \"Grid pans and zooms with content\"
      ]
      return_guarantees: []
    }
    invariants: [
      \"Grid dots remain crisp at all zoom levels\",
      \"Grid does not interfere with node interaction\"
    ]
  }

  research_requirements: {
    files_to_read: [
      {path: \"busted-flow/components/flow/flow-canvas.tsx:230-235\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"ui/minimap.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"}
    ]
    research_questions: [
      {question: \"Should grid be CSS background or SVG overlay?\", answered: false}
    ]
    research_complete_when: [
      "All files have been read and patterns documented"
    ]
  }

  inversions: {
    usability_failures: [
      {failure: "User encounters unclear error", prevention: "Provide specific error messages", test_for_it: "test_error_messages_are_clear"}
    ]
  }

  acceptance_tests: {
    happy_paths: [
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"},
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"}
    ]
    error_paths: [
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"},
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"}
    ]
  }

  e2e_tests: {
    pipeline_test: {
      name: "test_full_pipeline"
      description: "End-to-end test of full workflow"
      setup: {}
      execute: {
        command: "intent command"
      }
      verify: {
        exit_code: 0
      }
    }
  }

  verification_checkpoints: {
    gate_0_research: {
      name: "Research Gate"
      must_pass_before: "Writing code"
      checks: ["All research questions answered"]
      evidence_required: ["Research notes documented"]
    }
    gate_1_tests: {
      name: "Test Gate"
      must_pass_before: "Implementation"
      checks: ["All tests written and failing"]
      evidence_required: ["Test files exist"]
    }
    gate_2_implementation: {
      name: "Implementation Gate"
      must_pass_before: "Completion"
      checks: ["All tests pass"]
      evidence_required: ["CI green"]
    }
    gate_3_integration: {
      name: "Integration Gate"
      must_pass_before: "Closing bead"
      checks: ["E2E tests pass"]
      evidence_required: ["Manual verification complete"]
    }
  }

  implementation_tasks: {
    phase_0_research: {
      parallelizable: true
      tasks: [
        {task: \"Review busted-flow dot grid implementation at lines 230-235\", done_when: \"Documented\", parallel_group: \"research\"},
        {task: \"Decide between CSS background vs SVG overlay approach\", done_when: \"Documented\", parallel_group: \"research\"}
      ]
    }
    phase_1_tests_first: {
      parallelizable: true
      gate_required: "gate_0_research"
      tasks: [
        {task: \"Write visual test for grid rendering\", done_when: \"Test exists and fails\", parallel_group: \"tests\"},
        {task: \"Write test for grid scaling behavior\", done_when: \"Test exists and fails\", parallel_group: \"tests\"}
      ]
    }
    phase_2_implementation: {
      parallelizable: false
      gate_required: "gate_1_tests"
      tasks: [
        {task: \"Add GridOverlay component using CSS radial-gradient\", done_when: \"Tests pass\"},
        {task: \"Wire GridOverlay to viewport zoom and pan signals\", done_when: \"Tests pass\"},
        {task: \"Add grid visibility toggle to toolbar\", done_when: \"Tests pass\"}
      ]
    }
    phase_4_verification: {
      parallelizable: true
      gate_required: "gate_2_implementation"
      tasks: [
        {task: "Run moon run :ci", done_when: "CI passes", parallel_group: "verification"}
      ]
    }
  }

  failure_modes: {
    failure_modes: [
      {symptom: "Feature does not work", likely_cause: "Implementation incomplete", where_to_look: [{file: "src/main.rs", what_to_check: "Implementation logic"}], fix_pattern: "Complete implementation"}
    ]
  }

  anti_hallucination: {
    read_before_write: [
      {file: "src/main.rs", must_read_first: true, key_sections_to_understand: ["Main entry point"]}
    ]
    apis_that_exist: []
    no_placeholder_values: ["Use real data from codebase"]
    git_verification: {
      before_claiming_done: "git status && git diff && moon run :test"
    }
  }

  context_survival: {
    progress_file: {
      path: ".bead-progress/Seshat-20260223103614-rh1ocyvg/progress.txt"
      format: "Markdown checklist"
    }
    recovery_instructions: "Read progress.txt and continue from current task"
  }

  completion_checklist: {
    tests: [
      "[ ] All acceptance tests written and passing",
      "[ ] All error path tests written and passing",
      "[ ] E2E pipeline test passing with real data",
      "[ ] No mocks or fake data in any test"
    ]
    code: [
      "[ ] Implementation uses Result<T, Error> throughout",
      "[ ] Zero unwrap or expect calls"
    ]
    ci: [
      "[ ] moon run :ci passes"
    ]
  }

  context: {
    related_files: [
      {path: \"busted-flow/components/flow/flow-canvas.tsx:230-235\", relevance: \"Related implementation\"},
      {path: \"ui/toolbar.rs\", relevance: \"Related implementation\"}
    ]
    similar_implementations: [
      \"busted-flow uses backgroundImage with radial-gradient\"
    ]
  }

  ai_hints: {
    do: [
      "Use functional patterns: map, and_then, ?",
      "Return Result<T, Error> from all fallible functions",
      "READ files before modifying them"
    ]
    do_not: [
      "Do NOT use unwrap or expect",
      "Do NOT use panic!, todo!, or unimplemented!",
      "Do NOT modify clippy configuration"
    ]
    constitution: [
      "Zero unwrap law: NEVER use .unwrap or .expect",
      "Test first: Tests MUST exist before implementation"
    ]
  }
}
', NULL, NULL, NULL, 'closed', 1, 'feature', 'self', NULL, NULL, '2026-02-23T16:36:15.279650570+00:00', 'lewis', '2026-02-28T19:49:00.221549157+00:00', '2026-02-28T19:49:00.221533877+00:00', 'done', NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(244, 244, 38, 128, 'bd-rg7', '23f5c180367975e0354e7475a5021bcdbb8fe1aef97c7452f1b3e474c741a506', 'tests: Implement CLP clipboard tests', 'Add 10 clipboard tests: copy/paste single node, copy/paste multiple nodes with edges, copy/paste group structure, cut/paste removes original, duplicate shortcut, paste into container, drag-drop external image, clipboard serialization.', NULL, NULL, NULL, 'open', 2, 'task', NULL, NULL, NULL, '2026-03-01T00:39:14.139846248+00:00', 'lewis', '2026-03-01T00:39:14.139846248+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(244, 244, 38, 129, 'bd-rn3', '08e27ca42287e478acb20cf6ebf41b1e793d792669d17fb6f45ca69a3c7b81f7', 'tests: Implement SEL selection tests 4/5', 'Add 5 selection tests: multi-type selection (shape+text+connector), selection persists across pan/zoom, selection box after undo/redo, double-click enters edit mode.', NULL, NULL, NULL, 'open', 1, 'task', NULL, NULL, NULL, '2026-03-01T00:38:23.299234161+00:00', 'lewis', '2026-03-01T00:38:23.299234161+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(244, 244, 38, 130, 'bd-rpv', '692c336504b60f569183cc544aa4a32aa930bf3079f21f1566011bfaf2390e60', 'canvas: Box select with drag marquee', '# CUE Validation Schema
# Validate implementation: cue vet /home/lewis/src/whiteboard_kit/.beads/schemas/Seshat-20260223103614-qaipkmsj.cue implementation.cue
# Schema location: /home/lewis/src/whiteboard_kit/.beads/schemas/Seshat-20260223103614-qaipkmsj.cue


#EnhancedBead: {
  id: "Seshat-20260223103614-qaipkmsj"
  title: "canvas: Box select with drag marquee"
  type: "feature"
  priority: 1
  effort_estimate: "1hr"
  labels: ["planner-generated"]

  clarifications: {
    clarification_status: "RESOLVED"
  }

  ears_requirements: {
    ubiquitous: [
      \"THE SYSTEM SHALL display a selection rectangle while dragging on canvas\",
      \"THE SYSTEM SHALL select all nodes fully contained within the marquee\"
    ]
    event_driven: [
      {trigger: \"WHEN user drags on empty canvas\", shall: \"THE SYSTEM SHALL start marquee selection mode and draw rectangle\"},
      {trigger: \"WHEN user releases mouse after marquee drag\", shall: \"THE SYSTEM SHALL select all nodes within rectangle bounds\"}
    ]
    unwanted: [
      {condition: \"IF marquee contains no nodes\", shall_not: \"THE SYSTEM SHALL NOT clear existing selection\", because: \"Accidental empty marquee should preserve work\"}
    ]
  }

  contracts: {
    preconditions: {
      auth_required: false
      required_inputs: []
      system_state: [
        \"Canvas has marquee interaction mode defined\",
        \"Selection state supports multiple nodes\"
      ]
    }
    postconditions: {
      state_changes: [
        \"Marquee rectangle is cleared after selection\",
        \"All nodes within bounds are selected\"
      ]
      return_guarantees: []
    }
    invariants: [
      \"Marquee coordinates are in canvas space\",
      \"Partial overlap nodes are not selected\"
    ]
  }

  research_requirements: {
    files_to_read: [
      {path: \"hooks/use_canvas_interaction.rs:19-27\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"hooks/use_canvas_interaction.rs:135-145\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"busted-flow/components/flow/flow-canvas.tsx\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"}
    ]
    research_questions: [
      {question: \"Is marquee mode already implemented in use_canvas_interaction?\", answered: false}
    ]
    research_complete_when: [
      "All files have been read and patterns documented"
    ]
  }

  inversions: {
    usability_failures: [
      {failure: "User encounters unclear error", prevention: "Provide specific error messages", test_for_it: "test_error_messages_are_clear"}
    ]
  }

  acceptance_tests: {
    happy_paths: [
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"},
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"}
    ]
    error_paths: [
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"},
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"}
    ]
  }

  e2e_tests: {
    pipeline_test: {
      name: "test_full_pipeline"
      description: "End-to-end test of full workflow"
      setup: {}
      execute: {
        command: "intent command"
      }
      verify: {
        exit_code: 0
      }
    }
  }

  verification_checkpoints: {
    gate_0_research: {
      name: "Research Gate"
      must_pass_before: "Writing code"
      checks: ["All research questions answered"]
      evidence_required: ["Research notes documented"]
    }
    gate_1_tests: {
      name: "Test Gate"
      must_pass_before: "Implementation"
      checks: ["All tests written and failing"]
      evidence_required: ["Test files exist"]
    }
    gate_2_implementation: {
      name: "Implementation Gate"
      must_pass_before: "Completion"
      checks: ["All tests pass"]
      evidence_required: ["CI green"]
    }
    gate_3_integration: {
      name: "Integration Gate"
      must_pass_before: "Closing bead"
      checks: ["E2E tests pass"]
      evidence_required: ["Manual verification complete"]
    }
  }

  implementation_tasks: {
    phase_0_research: {
      parallelizable: true
      tasks: [
        {task: \"Review use_canvas_interaction.rs Marquee variant\", done_when: \"Documented\", parallel_group: \"research\"},
        {task: \"Check if visual marquee component exists\", done_when: \"Documented\", parallel_group: \"research\"}
      ]
    }
    phase_1_tests_first: {
      parallelizable: true
      gate_required: "gate_0_research"
      tasks: [
        {task: \"Write test for marquee bounds calculation\", done_when: \"Test exists and fails\", parallel_group: \"tests\"},
        {task: \"Write test for node containment check\", done_when: \"Test exists and fails\", parallel_group: \"tests\"}
      ]
    }
    phase_2_implementation: {
      parallelizable: false
      gate_required: "gate_1_tests"
      tasks: [
        {task: \"Add MarqueeOverlay SVG component for visual rectangle\", done_when: \"Tests pass\"},
        {task: \"Implement node filtering by marquee bounds\", done_when: \"Tests pass\"},
        {task: \"Wire marquee complete to selection update\", done_when: \"Tests pass\"}
      ]
    }
    phase_4_verification: {
      parallelizable: true
      gate_required: "gate_2_implementation"
      tasks: [
        {task: "Run moon run :ci", done_when: "CI passes", parallel_group: "verification"}
      ]
    }
  }

  failure_modes: {
    failure_modes: [
      {symptom: "Feature does not work", likely_cause: "Implementation incomplete", where_to_look: [{file: "src/main.rs", what_to_check: "Implementation logic"}], fix_pattern: "Complete implementation"}
    ]
  }

  anti_hallucination: {
    read_before_write: [
      {file: "src/main.rs", must_read_first: true, key_sections_to_understand: ["Main entry point"]}
    ]
    apis_that_exist: []
    no_placeholder_values: ["Use real data from codebase"]
    git_verification: {
      before_claiming_done: "git status && git diff && moon run :test"
    }
  }

  context_survival: {
    progress_file: {
      path: ".bead-progress/Seshat-20260223103614-qaipkmsj/progress.txt"
      format: "Markdown checklist"
    }
    recovery_instructions: "Read progress.txt and continue from current task"
  }

  completion_checklist: {
    tests: [
      "[ ] All acceptance tests written and passing",
      "[ ] All error path tests written and passing",
      "[ ] E2E pipeline test passing with real data",
      "[ ] No mocks or fake data in any test"
    ]
    code: [
      "[ ] Implementation uses Result<T, Error> throughout",
      "[ ] Zero unwrap or expect calls"
    ]
    ci: [
      "[ ] moon run :ci passes"
    ]
  }

  context: {
    related_files: [
      {path: \"hooks/use_canvas_interaction.rs:19-27\", relevance: \"Related implementation\"},
      {path: \"hooks/use_canvas_interaction.rs:135-145\", relevance: \"Related implementation\"}
    ]
    similar_implementations: [
      \"InteractionMode::Marquee already exists in use_canvas_interaction\"
    ]
  }

  ai_hints: {
    do: [
      "Use functional patterns: map, and_then, ?",
      "Return Result<T, Error> from all fallible functions",
      "READ files before modifying them"
    ]
    do_not: [
      "Do NOT use unwrap or expect",
      "Do NOT use panic!, todo!, or unimplemented!",
      "Do NOT modify clippy configuration"
    ]
    constitution: [
      "Zero unwrap law: NEVER use .unwrap or .expect",
      "Test first: Tests MUST exist before implementation"
    ]
  }
}
', NULL, NULL, NULL, 'closed', 1, 'feature', 'self', NULL, NULL, '2026-02-23T16:36:15.384498306+00:00', 'lewis', '2026-02-28T19:49:00.859807805+00:00', '2026-02-28T19:49:00.859790805+00:00', 'done', NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(244, 244, 38, 131, 'bd-s62', 'fb020749d77aaf5f65f67b59ebea967de5191aef8ff1e7cc9740efafe6b33a9e', 'conflict-human-priority: reject conflicting ai ops during active human edits', '# CUE Validation Schema
# Validate implementation: cue vet /home/lewis/src/seshat/.beads/schemas/Seshat-20260227010334-ngf4bqmy.cue implementation.cue
# Schema location: /home/lewis/src/seshat/.beads/schemas/Seshat-20260227010334-ngf4bqmy.cue


#EnhancedBead: {
  id: "Seshat-20260227010334-ngf4bqmy"
  title: "conflict-human-priority: reject conflicting ai ops during active human edits"
  type: "task"
  priority: 0
  effort_estimate: "30min"
  labels: ["planner-generated"]

  clarifications: {
    clarification_status: "RESOLVED"
  }

  ears_requirements: {
    ubiquitous: [
      \"THE SYSTEM SHALL use a hard-cutover rewrite with no legacy compatibility layer\"
    ]
    event_driven: [
      {trigger: \"WHEN a backend change is implemented\", shall: \"THE SYSTEM SHALL remove conflicting legacy behavior before enabling replacement behavior\"}
    ]
    unwanted: [
      {condition: \"IF legacy and new backends coexist in execution paths\", shall_not: \"THE SYSTEM SHALL NOT permit dual-write or fallback migration behavior\", because: \"dual paths create hidden divergence and higher defect risk\"}
    ]
  }

  contracts: {
    preconditions: {
      auth_required: false
      required_inputs: []
      system_state: [
        \"Rust Contract Signature: fn evaluate_human_priority(op: &EventEnvelope, state: &ProjectionState) -> Result<ConflictDecision, ConflictError>\",
        \"Rust Error Contract: enum ConflictError { HumanPriorityBlock, MissingEntity, PolicyViolation }\",
        \"Legacy code path for this slice is identified and removable in one commit\"
      ]
    }
    postconditions: {
      state_changes: [
        \"Rust Postcondition Signature: fn record_conflict_rejection(op: &EventEnvelope, reason: ConflictError) -> Result<(), ConflictError>\",
        \"Legacy path is deleted or unreachable by compile-time guarantees\",
        \"Replacement path passes focused tests with no fallback to removed code\"
      ]
      return_guarantees: []
    }
    invariants: [
      \"No migration path is introduced\",
      \"No dual-write compatibility path exists\",
      \"All fallible operations use typed Result errors\"
    ]
  }

  research_requirements: {
    files_to_read: [
      {path: \"diagram_tool/src/backend.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"diagram_tool/src/patch.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"diagram_tool/src/cli.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"diagram_tool/src/models/document.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"}
    ]
    research_questions: [
      {question: \"which symbol removals guarantee hard cutover for this slice\", answered: false}
    ]
    research_complete_when: [
      "All files have been read and patterns documented"
    ]
  }

  inversions: {
    usability_failures: [
      {failure: "User encounters unclear error", prevention: "Provide specific error messages", test_for_it: "test_error_messages_are_clear"}
    ]
  }

  acceptance_tests: {
    happy_paths: [
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"},
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"}
    ]
    error_paths: [
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"},
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"}
    ]
  }

  e2e_tests: {
    pipeline_test: {
      name: "test_full_pipeline"
      description: "End-to-end test of full workflow"
      setup: {}
      execute: {
        command: "intent command"
      }
      verify: {
        exit_code: 0
      }
    }
  }

  verification_checkpoints: {
    gate_0_research: {
      name: "Research Gate"
      must_pass_before: "Writing code"
      checks: ["All research questions answered"]
      evidence_required: ["Research notes documented"]
    }
    gate_1_tests: {
      name: "Test Gate"
      must_pass_before: "Implementation"
      checks: ["All tests written and failing"]
      evidence_required: ["Test files exist"]
    }
    gate_2_implementation: {
      name: "Implementation Gate"
      must_pass_before: "Completion"
      checks: ["All tests pass"]
      evidence_required: ["CI green"]
    }
    gate_3_integration: {
      name: "Integration Gate"
      must_pass_before: "Closing bead"
      checks: ["E2E tests pass"]
      evidence_required: ["Manual verification complete"]
    }
  }

  implementation_tasks: {
    phase_0_research: {
      parallelizable: true
      tasks: [
        {task: \"Read and mark exact legacy symbols to remove\", done_when: \"Documented\", parallel_group: \"research\"}
      ]
    }
    phase_1_tests_first: {
      parallelizable: true
      gate_required: "gate_0_research"
      tasks: [
        {task: \"Write failing tests that assert new-only behavior\", done_when: \"Test exists and fails\", parallel_group: \"tests\"}
      ]
    }
    phase_2_implementation: {
      parallelizable: false
      gate_required: "gate_1_tests"
      tasks: [
        {task: \"Track active human edit windows by entity\", done_when: \"Tests pass\"},
        {task: \"Emit deterministic HUMAN_PRIORITY_BLOCK rejections\", done_when: \"Tests pass\"}
      ]
    }
    phase_4_verification: {
      parallelizable: true
      gate_required: "gate_2_implementation"
      tasks: [
        {task: "Run moon run :ci", done_when: "CI passes", parallel_group: "verification"}
      ]
    }
  }

  failure_modes: {
    failure_modes: [
      {symptom: "Feature does not work", likely_cause: "Implementation incomplete", where_to_look: [{file: "src/main.rs", what_to_check: "Implementation logic"}], fix_pattern: "Complete implementation"}
    ]
  }

  anti_hallucination: {
    read_before_write: [
      {file: "src/main.rs", must_read_first: true, key_sections_to_understand: ["Main entry point"]}
    ]
    apis_that_exist: []
    no_placeholder_values: ["Use real data from codebase"]
    git_verification: {
      before_claiming_done: "git status && git diff && moon run :test"
    }
  }

  context_survival: {
    progress_file: {
      path: ".bead-progress/Seshat-20260227010334-ngf4bqmy/progress.txt"
      format: "Markdown checklist"
    }
    recovery_instructions: "Read progress.txt and continue from current task"
  }

  completion_checklist: {
    tests: [
      "[ ] All acceptance tests written and passing",
      "[ ] All error path tests written and passing",
      "[ ] E2E pipeline test passing with real data",
      "[ ] No mocks or fake data in any test"
    ]
    code: [
      "[ ] Implementation uses Result<T, Error> throughout",
      "[ ] Zero unwrap or expect calls"
    ]
    ci: [
      "[ ] moon run :ci passes"
    ]
  }

  context: {
    related_files: [
      {path: \"diagram_tool/src/backend.rs\", relevance: \"Related implementation\"},
      {path: \"diagram_tool/src/patch.rs\", relevance: \"Related implementation\"},
      {path: \"diagram_tool/src/cli.rs\", relevance: \"Related implementation\"},
      {path: \"diagram_tool/src/models/document.rs\", relevance: \"Related implementation\"}
    ]
    similar_implementations: [
      \"existing strict error mapping in CLI commands\"
    ]
  }

  ai_hints: {
    do: [
      "Use functional patterns: map, and_then, ?",
      "Return Result<T, Error> from all fallible functions",
      "READ files before modifying them"
    ]
    do_not: [
      "Do NOT use unwrap or expect",
      "Do NOT use panic!, todo!, or unimplemented!",
      "Do NOT modify clippy configuration"
    ]
    constitution: [
      "Zero unwrap law: NEVER use .unwrap or .expect",
      "Test first: Tests MUST exist before implementation"
    ]
  }
}
', NULL, NULL, NULL, 'open', 0, 'task', NULL, NULL, NULL, '2026-02-27T07:03:36.490928391+00:00', 'lewis', '2026-02-27T07:03:59.104268775+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(244, 244, 38, 132, 'bd-sa6', '53a40bf88ab731697a365763c6fac77b3a021a1c8c16665c6da3e7e4a2665ef7', 'tests: Implement SUB subgraph tests 4/4', 'Add 5 subgraph tests: click inside container selects child vs container with modifier, box-select across container, collapse/expand container, locked container with unlocked children interactions.', NULL, NULL, NULL, 'open', 1, 'task', NULL, NULL, NULL, '2026-03-01T00:39:01.742414244+00:00', 'lewis', '2026-03-01T00:39:01.742414244+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(250, 250, 38, 128, 'bd-rg7', '23f5c180367975e0354e7475a5021bcdbb8fe1aef97c7452f1b3e474c741a506', 'tests: Implement CLP clipboard tests', 'Add 10 clipboard tests: copy/paste single node, copy/paste multiple nodes with edges, copy/paste group structure, cut/paste removes original, duplicate shortcut, paste into container, drag-drop external image, clipboard serialization.', NULL, NULL, NULL, 'open', 2, 'task', NULL, NULL, NULL, '2026-03-01T00:39:14.139846248+00:00', 'lewis', '2026-03-01T00:39:14.139846248+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(250, 250, 38, 129, 'bd-rn3', '08e27ca42287e478acb20cf6ebf41b1e793d792669d17fb6f45ca69a3c7b81f7', 'tests: Implement SEL selection tests 4/5', 'Add 5 selection tests: multi-type selection (shape+text+connector), selection persists across pan/zoom, selection box after undo/redo, double-click enters edit mode.', NULL, NULL, NULL, 'open', 1, 'task', NULL, NULL, NULL, '2026-03-01T00:38:23.299234161+00:00', 'lewis', '2026-03-01T00:38:23.299234161+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(250, 250, 38, 130, 'bd-rpv', '692c336504b60f569183cc544aa4a32aa930bf3079f21f1566011bfaf2390e60', 'canvas: Box select with drag marquee', '# CUE Validation Schema
# Validate implementation: cue vet /home/lewis/src/whiteboard_kit/.beads/schemas/Seshat-20260223103614-qaipkmsj.cue implementation.cue
# Schema location: /home/lewis/src/whiteboard_kit/.beads/schemas/Seshat-20260223103614-qaipkmsj.cue


#EnhancedBead: {
  id: "Seshat-20260223103614-qaipkmsj"
  title: "canvas: Box select with drag marquee"
  type: "feature"
  priority: 1
  effort_estimate: "1hr"
  labels: ["planner-generated"]

  clarifications: {
    clarification_status: "RESOLVED"
  }

  ears_requirements: {
    ubiquitous: [
      \"THE SYSTEM SHALL display a selection rectangle while dragging on canvas\",
      \"THE SYSTEM SHALL select all nodes fully contained within the marquee\"
    ]
    event_driven: [
      {trigger: \"WHEN user drags on empty canvas\", shall: \"THE SYSTEM SHALL start marquee selection mode and draw rectangle\"},
      {trigger: \"WHEN user releases mouse after marquee drag\", shall: \"THE SYSTEM SHALL select all nodes within rectangle bounds\"}
    ]
    unwanted: [
      {condition: \"IF marquee contains no nodes\", shall_not: \"THE SYSTEM SHALL NOT clear existing selection\", because: \"Accidental empty marquee should preserve work\"}
    ]
  }

  contracts: {
    preconditions: {
      auth_required: false
      required_inputs: []
      system_state: [
        \"Canvas has marquee interaction mode defined\",
        \"Selection state supports multiple nodes\"
      ]
    }
    postconditions: {
      state_changes: [
        \"Marquee rectangle is cleared after selection\",
        \"All nodes within bounds are selected\"
      ]
      return_guarantees: []
    }
    invariants: [
      \"Marquee coordinates are in canvas space\",
      \"Partial overlap nodes are not selected\"
    ]
  }

  research_requirements: {
    files_to_read: [
      {path: \"hooks/use_canvas_interaction.rs:19-27\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"hooks/use_canvas_interaction.rs:135-145\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"busted-flow/components/flow/flow-canvas.tsx\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"}
    ]
    research_questions: [
      {question: \"Is marquee mode already implemented in use_canvas_interaction?\", answered: false}
    ]
    research_complete_when: [
      "All files have been read and patterns documented"
    ]
  }

  inversions: {
    usability_failures: [
      {failure: "User encounters unclear error", prevention: "Provide specific error messages", test_for_it: "test_error_messages_are_clear"}
    ]
  }

  acceptance_tests: {
    happy_paths: [
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"},
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"}
    ]
    error_paths: [
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"},
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"}
    ]
  }

  e2e_tests: {
    pipeline_test: {
      name: "test_full_pipeline"
      description: "End-to-end test of full workflow"
      setup: {}
      execute: {
        command: "intent command"
      }
      verify: {
        exit_code: 0
      }
    }
  }

  verification_checkpoints: {
    gate_0_research: {
      name: "Research Gate"
      must_pass_before: "Writing code"
      checks: ["All research questions answered"]
      evidence_required: ["Research notes documented"]
    }
    gate_1_tests: {
      name: "Test Gate"
      must_pass_before: "Implementation"
      checks: ["All tests written and failing"]
      evidence_required: ["Test files exist"]
    }
    gate_2_implementation: {
      name: "Implementation Gate"
      must_pass_before: "Completion"
      checks: ["All tests pass"]
      evidence_required: ["CI green"]
    }
    gate_3_integration: {
      name: "Integration Gate"
      must_pass_before: "Closing bead"
      checks: ["E2E tests pass"]
      evidence_required: ["Manual verification complete"]
    }
  }

  implementation_tasks: {
    phase_0_research: {
      parallelizable: true
      tasks: [
        {task: \"Review use_canvas_interaction.rs Marquee variant\", done_when: \"Documented\", parallel_group: \"research\"},
        {task: \"Check if visual marquee component exists\", done_when: \"Documented\", parallel_group: \"research\"}
      ]
    }
    phase_1_tests_first: {
      parallelizable: true
      gate_required: "gate_0_research"
      tasks: [
        {task: \"Write test for marquee bounds calculation\", done_when: \"Test exists and fails\", parallel_group: \"tests\"},
        {task: \"Write test for node containment check\", done_when: \"Test exists and fails\", parallel_group: \"tests\"}
      ]
    }
    phase_2_implementation: {
      parallelizable: false
      gate_required: "gate_1_tests"
      tasks: [
        {task: \"Add MarqueeOverlay SVG component for visual rectangle\", done_when: \"Tests pass\"},
        {task: \"Implement node filtering by marquee bounds\", done_when: \"Tests pass\"},
        {task: \"Wire marquee complete to selection update\", done_when: \"Tests pass\"}
      ]
    }
    phase_4_verification: {
      parallelizable: true
      gate_required: "gate_2_implementation"
      tasks: [
        {task: "Run moon run :ci", done_when: "CI passes", parallel_group: "verification"}
      ]
    }
  }

  failure_modes: {
    failure_modes: [
      {symptom: "Feature does not work", likely_cause: "Implementation incomplete", where_to_look: [{file: "src/main.rs", what_to_check: "Implementation logic"}], fix_pattern: "Complete implementation"}
    ]
  }

  anti_hallucination: {
    read_before_write: [
      {file: "src/main.rs", must_read_first: true, key_sections_to_understand: ["Main entry point"]}
    ]
    apis_that_exist: []
    no_placeholder_values: ["Use real data from codebase"]
    git_verification: {
      before_claiming_done: "git status && git diff && moon run :test"
    }
  }

  context_survival: {
    progress_file: {
      path: ".bead-progress/Seshat-20260223103614-qaipkmsj/progress.txt"
      format: "Markdown checklist"
    }
    recovery_instructions: "Read progress.txt and continue from current task"
  }

  completion_checklist: {
    tests: [
      "[ ] All acceptance tests written and passing",
      "[ ] All error path tests written and passing",
      "[ ] E2E pipeline test passing with real data",
      "[ ] No mocks or fake data in any test"
    ]
    code: [
      "[ ] Implementation uses Result<T, Error> throughout",
      "[ ] Zero unwrap or expect calls"
    ]
    ci: [
      "[ ] moon run :ci passes"
    ]
  }

  context: {
    related_files: [
      {path: \"hooks/use_canvas_interaction.rs:19-27\", relevance: \"Related implementation\"},
      {path: \"hooks/use_canvas_interaction.rs:135-145\", relevance: \"Related implementation\"}
    ]
    similar_implementations: [
      \"InteractionMode::Marquee already exists in use_canvas_interaction\"
    ]
  }

  ai_hints: {
    do: [
      "Use functional patterns: map, and_then, ?",
      "Return Result<T, Error> from all fallible functions",
      "READ files before modifying them"
    ]
    do_not: [
      "Do NOT use unwrap or expect",
      "Do NOT use panic!, todo!, or unimplemented!",
      "Do NOT modify clippy configuration"
    ]
    constitution: [
      "Zero unwrap law: NEVER use .unwrap or .expect",
      "Test first: Tests MUST exist before implementation"
    ]
  }
}
', NULL, NULL, NULL, 'closed', 1, 'feature', 'self', NULL, NULL, '2026-02-23T16:36:15.384498306+00:00', 'lewis', '2026-02-28T19:49:00.859807805+00:00', '2026-02-28T19:49:00.859790805+00:00', 'done', NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(250, 250, 38, 131, 'bd-s62', 'fb020749d77aaf5f65f67b59ebea967de5191aef8ff1e7cc9740efafe6b33a9e', 'conflict-human-priority: reject conflicting ai ops during active human edits', '# CUE Validation Schema
# Validate implementation: cue vet /home/lewis/src/seshat/.beads/schemas/Seshat-20260227010334-ngf4bqmy.cue implementation.cue
# Schema location: /home/lewis/src/seshat/.beads/schemas/Seshat-20260227010334-ngf4bqmy.cue


#EnhancedBead: {
  id: "Seshat-20260227010334-ngf4bqmy"
  title: "conflict-human-priority: reject conflicting ai ops during active human edits"
  type: "task"
  priority: 0
  effort_estimate: "30min"
  labels: ["planner-generated"]

  clarifications: {
    clarification_status: "RESOLVED"
  }

  ears_requirements: {
    ubiquitous: [
      \"THE SYSTEM SHALL use a hard-cutover rewrite with no legacy compatibility layer\"
    ]
    event_driven: [
      {trigger: \"WHEN a backend change is implemented\", shall: \"THE SYSTEM SHALL remove conflicting legacy behavior before enabling replacement behavior\"}
    ]
    unwanted: [
      {condition: \"IF legacy and new backends coexist in execution paths\", shall_not: \"THE SYSTEM SHALL NOT permit dual-write or fallback migration behavior\", because: \"dual paths create hidden divergence and higher defect risk\"}
    ]
  }

  contracts: {
    preconditions: {
      auth_required: false
      required_inputs: []
      system_state: [
        \"Rust Contract Signature: fn evaluate_human_priority(op: &EventEnvelope, state: &ProjectionState) -> Result<ConflictDecision, ConflictError>\",
        \"Rust Error Contract: enum ConflictError { HumanPriorityBlock, MissingEntity, PolicyViolation }\",
        \"Legacy code path for this slice is identified and removable in one commit\"
      ]
    }
    postconditions: {
      state_changes: [
        \"Rust Postcondition Signature: fn record_conflict_rejection(op: &EventEnvelope, reason: ConflictError) -> Result<(), ConflictError>\",
        \"Legacy path is deleted or unreachable by compile-time guarantees\",
        \"Replacement path passes focused tests with no fallback to removed code\"
      ]
      return_guarantees: []
    }
    invariants: [
      \"No migration path is introduced\",
      \"No dual-write compatibility path exists\",
      \"All fallible operations use typed Result errors\"
    ]
  }

  research_requirements: {
    files_to_read: [
      {path: \"diagram_tool/src/backend.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"diagram_tool/src/patch.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"diagram_tool/src/cli.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"},
      {path: \"diagram_tool/src/models/document.rs\", what_to_extract: \"Existing patterns\", document_in: \"research_notes.md\"}
    ]
    research_questions: [
      {question: \"which symbol removals guarantee hard cutover for this slice\", answered: false}
    ]
    research_complete_when: [
      "All files have been read and patterns documented"
    ]
  }

  inversions: {
    usability_failures: [
      {failure: "User encounters unclear error", prevention: "Provide specific error messages", test_for_it: "test_error_messages_are_clear"}
    ]
  }

  acceptance_tests: {
    happy_paths: [
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"},
      {name: \"test_happy_path\", given: \"Valid inputs\", when: \"User executes command\", then: [\"Exit code is 0\", \"Output is correct\"], real_input: \"command input\", expected_output: \"expected output\"}
    ]
    error_paths: [
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"},
      {name: \"test_error_path\", given: \"Invalid inputs\", when: \"User executes command\", then: [\"Exit code is non-zero\", \"Error message is clear\"], real_input: \"invalid input\", expected_output: null, expected_error: \"error message\"}
    ]
  }

  e2e_tests: {
    pipeline_test: {
      name: "test_full_pipeline"
      description: "End-to-end test of full workflow"
      setup: {}
      execute: {
        command: "intent command"
      }
      verify: {
        exit_code: 0
      }
    }
  }

  verification_checkpoints: {
    gate_0_research: {
      name: "Research Gate"
      must_pass_before: "Writing code"
      checks: ["All research questions answered"]
      evidence_required: ["Research notes documented"]
    }
    gate_1_tests: {
      name: "Test Gate"
      must_pass_before: "Implementation"
      checks: ["All tests written and failing"]
      evidence_required: ["Test files exist"]
    }
    gate_2_implementation: {
      name: "Implementation Gate"
      must_pass_before: "Completion"
      checks: ["All tests pass"]
      evidence_required: ["CI green"]
    }
    gate_3_integration: {
      name: "Integration Gate"
      must_pass_before: "Closing bead"
      checks: ["E2E tests pass"]
      evidence_required: ["Manual verification complete"]
    }
  }

  implementation_tasks: {
    phase_0_research: {
      parallelizable: true
      tasks: [
        {task: \"Read and mark exact legacy symbols to remove\", done_when: \"Documented\", parallel_group: \"research\"}
      ]
    }
    phase_1_tests_first: {
      parallelizable: true
      gate_required: "gate_0_research"
      tasks: [
        {task: \"Write failing tests that assert new-only behavior\", done_when: \"Test exists and fails\", parallel_group: \"tests\"}
      ]
    }
    phase_2_implementation: {
      parallelizable: false
      gate_required: "gate_1_tests"
      tasks: [
        {task: \"Track active human edit windows by entity\", done_when: \"Tests pass\"},
        {task: \"Emit deterministic HUMAN_PRIORITY_BLOCK rejections\", done_when: \"Tests pass\"}
      ]
    }
    phase_4_verification: {
      parallelizable: true
      gate_required: "gate_2_implementation"
      tasks: [
        {task: "Run moon run :ci", done_when: "CI passes", parallel_group: "verification"}
      ]
    }
  }

  failure_modes: {
    failure_modes: [
      {symptom: "Feature does not work", likely_cause: "Implementation incomplete", where_to_look: [{file: "src/main.rs", what_to_check: "Implementation logic"}], fix_pattern: "Complete implementation"}
    ]
  }

  anti_hallucination: {
    read_before_write: [
      {file: "src/main.rs", must_read_first: true, key_sections_to_understand: ["Main entry point"]}
    ]
    apis_that_exist: []
    no_placeholder_values: ["Use real data from codebase"]
    git_verification: {
      before_claiming_done: "git status && git diff && moon run :test"
    }
  }

  context_survival: {
    progress_file: {
      path: ".bead-progress/Seshat-20260227010334-ngf4bqmy/progress.txt"
      format: "Markdown checklist"
    }
    recovery_instructions: "Read progress.txt and continue from current task"
  }

  completion_checklist: {
    tests: [
      "[ ] All acceptance tests written and passing",
      "[ ] All error path tests written and passing",
      "[ ] E2E pipeline test passing with real data",
      "[ ] No mocks or fake data in any test"
    ]
    code: [
      "[ ] Implementation uses Result<T, Error> throughout",
      "[ ] Zero unwrap or expect calls"
    ]
    ci: [
      "[ ] moon run :ci passes"
    ]
  }

  context: {
    related_files: [
      {path: \"diagram_tool/src/backend.rs\", relevance: \"Related implementation\"},
      {path: \"diagram_tool/src/patch.rs\", relevance: \"Related implementation\"},
      {path: \"diagram_tool/src/cli.rs\", relevance: \"Related implementation\"},
      {path: \"diagram_tool/src/models/document.rs\", relevance: \"Related implementation\"}
    ]
    similar_implementations: [
      \"existing strict error mapping in CLI commands\"
    ]
  }

  ai_hints: {
    do: [
      "Use functional patterns: map, and_then, ?",
      "Return Result<T, Error> from all fallible functions",
      "READ files before modifying them"
    ]
    do_not: [
      "Do NOT use unwrap or expect",
      "Do NOT use panic!, todo!, or unimplemented!",
      "Do NOT modify clippy configuration"
    ]
    constitution: [
      "Zero unwrap law: NEVER use .unwrap or .expect",
      "Test first: Tests MUST exist before implementation"
    ]
  }
}
', NULL, NULL, NULL, 'open', 0, 'task', NULL, NULL, NULL, '2026-02-27T07:03:36.490928391+00:00', 'lewis', '2026-02-27T07:03:59.104268775+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(250, 250, 38, 132, 'bd-sa6', '53a40bf88ab731697a365763c6fac77b3a021a1c8c16665c6da3e7e4a2665ef7', 'tests: Implement SUB subgraph tests 4/4', 'Add 5 subgraph tests: click inside container selects child vs container with modifier, box-select across container, collapse/expand container, locked container with unlocked children interactions.', NULL, NULL, NULL, 'open', 1, 'task', NULL, NULL, NULL, '2026-03-01T00:39:01.742414244+00:00', 'lewis', '2026-03-01T00:39:01.742414244+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, '.', NULL, NULL, NULL, NULL, 0, NULL, NULL, 0, NULL, 0, 0, 0, '.', 0);
INSERT INTO lost_and_found VALUES(260, 260, 3, 488, 'bd-1zz', '4e9582142ef8206174e639c554f985b8bd660568c4b3009177760267613b373c', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 489, 'bd-20k', '72f77028747a275f16e3c51afd7d0760c097164ff5208917bd1e655a94f5e3f8', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 490, 'bd-21j', '5e2cb53d2916c55fdc48dd6b180d4993162f539719e1076803305a0c7efade6f', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 491, 'bd-23z', 'c085111c7e926b552479f872f7cbf7e8fb7e8ed14c3cc60de6d4c1ec08ab4bd4', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 492, 'bd-24a', 'c0216f2acd8e54eac2374604e396266268ab9e92657e76b01cb0eae2a3cd0346', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 493, 'bd-25b', 'b9b5332bbd2dad69426a4b0e90ef00cf0de29f452450b00aa6dbb2ecd2887807', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 494, 'bd-260', '0a78706e3e8a434f4da9a2839028eb7598a1abf7de34e285b606e1b63893f99f', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 495, 'bd-26j', 'c8e8fd782a0c915c5d1003dd00079268d1da3ab198ba97bfb9365dcf6498258e', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 496, 'bd-275', '6d69f1968eae2f46350ebbf6112e93004caa872f2be7eb1fae5329e1f624de5a', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 497, 'bd-27q', '4ba815c310745c6e8609e76b378e371bb5f6418e49b37fd274d2bcd79ac43e15', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 498, 'bd-29g', '295ea1d9d7748918cecd2fbf2b3551b8252cf8b7c949c262f94f34f05e75ce77', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 499, 'bd-2b4', '0cd985a5c3e8d1a855eb91156d7b503aeaef847c5535d050945cb4e952e40386', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 500, 'bd-2b6', 'ed9c5b67e467d1b978d925928d320284c9a3eb0a4bf8509fdb78399ccd66b779', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 501, 'bd-2bh', '0d94138f76efccb4b2e23925e4cddd69bac853c648353163b752438ffc53532b', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 502, 'bd-2cb', '3406639168238754da1a9a766f48b35c7653ee7beafe61b52e55a307aaa586c6', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 503, 'bd-2cg', '0ffae1219bf876d95f991047b87b2ef1b4bad86964142f2571ecc127efdd0074', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 504, 'bd-2cm', '4ead30fa13162734972df934c2012e336a9c13ae7399f2d52e74cace40bd0afd', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 505, 'bd-2df', 'fdf8299b0ed621c9c00e43c4d5aeb704088b98257a7eb3f38882752ca06a567d', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 506, 'bd-2dg', 'beed3b123a4441215970b7708cf42226c1620da78517217891809df606d44c17', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 507, 'bd-2gl', '8016b819a0f1e8ab55b2f931091275f1acc1975fb07f009a206edc9ede40bf6f', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 508, 'bd-2hz', 'ad9e59d8844aa2086a5bddfbe95a087f62005c336dc6c8b6b7d305eb9c76171c', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 509, 'bd-2ii', '72c4a5ee7b47ec63873234221dd3a21ec4550e1000d04177b2c4cf7c97e7213c', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 510, 'bd-2ik', '7c0a32c088dd8d63efca91d6f51052db2f3af846d39684f5ff43dd743f68ad1f', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 511, 'bd-2jh', 'afafe4ac9c6d3b84bd4af572ce0f1afb01d4a1a8e9a85de233d6b891d9cbf0a0', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 512, 'bd-2jv', '83d40916ee7c712da821ffdb02688527d387a235a993bce0a659b0ed44bd00d5', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 513, 'bd-2kq', 'f800b7f9234207a14c1527eed0335cfaeb8067e04dfa2dfc81110452a2c0503a', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 514, 'bd-2kx', '925200b8025ce0bf7e4dc7d3d19ef702a8fef20cdc97d8c1a4cc639dff83a179', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 515, 'bd-2l6', 'fc16444c3463ee7d58069f79bf5e33e0d0223e79bba33d33b3dcd8c8d4228a7b', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 516, 'bd-2mk', '2d178c9b2b3b38cf89d5cf6d6d3e8e7378607f8907478ec16f657e9a08f7eee3', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 517, 'bd-2ng', 'd02177797689f28b08b816917b2adbcc09ac79ef206d37ef5ed34a6cba040e15', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 518, 'bd-2o1', '98b2e22428544fd18aa2e0b4dc1cee6e99403dd8af23d72ada42c07e044a3fe9', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 519, 'bd-2p8', '0560e35fa81d9a509dee4d67c5680665090bbcc62f3b2108a6fa5758089d13e9', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 520, 'bd-2qg', '1fc057db9d993d36cfd6f223cfb6a8d0cfb6699bb29964204e9047948619a39c', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 521, 'bd-2sa', '471778134cb28c79f28c0c9988e23b7acbe58dd96ca20b6ac7a872b9e436f358', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(260, 260, 3, 522, 'bd-2tx', 'e8d8156eb07d89a80af9e87a27a607e27483f3b30b36ff21cc5e4cad14411010', '2026-03-01T19:13:08.183737949+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 628, 'bd-21j', '5e2cb53d2916c55fdc48dd6b180d4993162f539719e1076803305a0c7efade6f', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 629, 'bd-23z', 'c085111c7e926b552479f872f7cbf7e8fb7e8ed14c3cc60de6d4c1ec08ab4bd4', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 630, 'bd-24a', 'c0216f2acd8e54eac2374604e396266268ab9e92657e76b01cb0eae2a3cd0346', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 631, 'bd-25b', 'b9b5332bbd2dad69426a4b0e90ef00cf0de29f452450b00aa6dbb2ecd2887807', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 632, 'bd-260', '0a78706e3e8a434f4da9a2839028eb7598a1abf7de34e285b606e1b63893f99f', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 633, 'bd-26j', 'c8e8fd782a0c915c5d1003dd00079268d1da3ab198ba97bfb9365dcf6498258e', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 634, 'bd-275', '6d69f1968eae2f46350ebbf6112e93004caa872f2be7eb1fae5329e1f624de5a', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 635, 'bd-27q', '4ba815c310745c6e8609e76b378e371bb5f6418e49b37fd274d2bcd79ac43e15', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 636, 'bd-29g', '295ea1d9d7748918cecd2fbf2b3551b8252cf8b7c949c262f94f34f05e75ce77', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 637, 'bd-2b4', '0cd985a5c3e8d1a855eb91156d7b503aeaef847c5535d050945cb4e952e40386', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 638, 'bd-2b6', 'ed9c5b67e467d1b978d925928d320284c9a3eb0a4bf8509fdb78399ccd66b779', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 639, 'bd-2bh', '0d94138f76efccb4b2e23925e4cddd69bac853c648353163b752438ffc53532b', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 640, 'bd-2cb', '3406639168238754da1a9a766f48b35c7653ee7beafe61b52e55a307aaa586c6', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 641, 'bd-2cg', '0ffae1219bf876d95f991047b87b2ef1b4bad86964142f2571ecc127efdd0074', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 642, 'bd-2cm', '4ead30fa13162734972df934c2012e336a9c13ae7399f2d52e74cace40bd0afd', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 643, 'bd-2df', 'fdf8299b0ed621c9c00e43c4d5aeb704088b98257a7eb3f38882752ca06a567d', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 644, 'bd-2dg', 'beed3b123a4441215970b7708cf42226c1620da78517217891809df606d44c17', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 645, 'bd-2gl', '8016b819a0f1e8ab55b2f931091275f1acc1975fb07f009a206edc9ede40bf6f', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 646, 'bd-2hz', 'ad9e59d8844aa2086a5bddfbe95a087f62005c336dc6c8b6b7d305eb9c76171c', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 647, 'bd-2ii', '72c4a5ee7b47ec63873234221dd3a21ec4550e1000d04177b2c4cf7c97e7213c', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 648, 'bd-2ik', '7c0a32c088dd8d63efca91d6f51052db2f3af846d39684f5ff43dd743f68ad1f', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 649, 'bd-2jh', 'afafe4ac9c6d3b84bd4af572ce0f1afb01d4a1a8e9a85de233d6b891d9cbf0a0', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 650, 'bd-2jv', '83d40916ee7c712da821ffdb02688527d387a235a993bce0a659b0ed44bd00d5', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 651, 'bd-2kq', 'f800b7f9234207a14c1527eed0335cfaeb8067e04dfa2dfc81110452a2c0503a', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 652, 'bd-2kx', '925200b8025ce0bf7e4dc7d3d19ef702a8fef20cdc97d8c1a4cc639dff83a179', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 653, 'bd-2l6', 'fc16444c3463ee7d58069f79bf5e33e0d0223e79bba33d33b3dcd8c8d4228a7b', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 654, 'bd-2mk', '2d178c9b2b3b38cf89d5cf6d6d3e8e7378607f8907478ec16f657e9a08f7eee3', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 655, 'bd-2ng', 'd02177797689f28b08b816917b2adbcc09ac79ef206d37ef5ed34a6cba040e15', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 656, 'bd-2o1', '98b2e22428544fd18aa2e0b4dc1cee6e99403dd8af23d72ada42c07e044a3fe9', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 657, 'bd-2p8', '0560e35fa81d9a509dee4d67c5680665090bbcc62f3b2108a6fa5758089d13e9', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 658, 'bd-2qg', '1fc057db9d993d36cfd6f223cfb6a8d0cfb6699bb29964204e9047948619a39c', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 659, 'bd-2sa', '471778134cb28c79f28c0c9988e23b7acbe58dd96ca20b6ac7a872b9e436f358', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 660, 'bd-2tx', 'e8d8156eb07d89a80af9e87a27a607e27483f3b30b36ff21cc5e4cad14411010', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 661, 'bd-2u3', '44342e53fbf735f0978a40686107bb35cbffaf1916ca6a80f77462ef72a68a82', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(261, 261, 3, 662, 'bd-2uy', 'b8667fd72f62a2ec28dcb4bee8cb709a85bdf879f590c2ff9c9f7a52e6fd971a', '2026-03-01T19:17:01.532415507+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 173, 'bd-20k', '5d75313e39699cfcc4e1d19bf1e95f33311b7fe67081545aaf7355a5d81485b0', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 174, 'bd-21j', '5e2cb53d2916c55fdc48dd6b180d4993162f539719e1076803305a0c7efade6f', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 175, 'bd-23z', 'c085111c7e926b552479f872f7cbf7e8fb7e8ed14c3cc60de6d4c1ec08ab4bd4', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 176, 'bd-24a', 'c0216f2acd8e54eac2374604e396266268ab9e92657e76b01cb0eae2a3cd0346', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 177, 'bd-25b', 'b9b5332bbd2dad69426a4b0e90ef00cf0de29f452450b00aa6dbb2ecd2887807', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 178, 'bd-260', '0a78706e3e8a434f4da9a2839028eb7598a1abf7de34e285b606e1b63893f99f', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 179, 'bd-26j', 'c8e8fd782a0c915c5d1003dd00079268d1da3ab198ba97bfb9365dcf6498258e', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 180, 'bd-275', '6d69f1968eae2f46350ebbf6112e93004caa872f2be7eb1fae5329e1f624de5a', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 181, 'bd-27q', '4ba815c310745c6e8609e76b378e371bb5f6418e49b37fd274d2bcd79ac43e15', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 182, 'bd-29g', '295ea1d9d7748918cecd2fbf2b3551b8252cf8b7c949c262f94f34f05e75ce77', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 183, 'bd-2b4', '0cd985a5c3e8d1a855eb91156d7b503aeaef847c5535d050945cb4e952e40386', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 184, 'bd-2b6', '46ead1a9b8033ad3f56811eed2785e807ef109e150c336ae16052255e715d299', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 185, 'bd-2bh', '0d94138f76efccb4b2e23925e4cddd69bac853c648353163b752438ffc53532b', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 186, 'bd-2cb', '3406639168238754da1a9a766f48b35c7653ee7beafe61b52e55a307aaa586c6', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 187, 'bd-2cg', '0ffae1219bf876d95f991047b87b2ef1b4bad86964142f2571ecc127efdd0074', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 188, 'bd-2cm', '4ead30fa13162734972df934c2012e336a9c13ae7399f2d52e74cace40bd0afd', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 189, 'bd-2df', 'fdf8299b0ed621c9c00e43c4d5aeb704088b98257a7eb3f38882752ca06a567d', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 190, 'bd-2dg', 'beed3b123a4441215970b7708cf42226c1620da78517217891809df606d44c17', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 191, 'bd-2gl', '8016b819a0f1e8ab55b2f931091275f1acc1975fb07f009a206edc9ede40bf6f', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 192, 'bd-2hz', 'ad9e59d8844aa2086a5bddfbe95a087f62005c336dc6c8b6b7d305eb9c76171c', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 193, 'bd-2ii', '72c4a5ee7b47ec63873234221dd3a21ec4550e1000d04177b2c4cf7c97e7213c', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 194, 'bd-2ik', '7c0a32c088dd8d63efca91d6f51052db2f3af846d39684f5ff43dd743f68ad1f', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 195, 'bd-2jh', 'afafe4ac9c6d3b84bd4af572ce0f1afb01d4a1a8e9a85de233d6b891d9cbf0a0', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 196, 'bd-2jv', '83d40916ee7c712da821ffdb02688527d387a235a993bce0a659b0ed44bd00d5', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 197, 'bd-2kq', 'f800b7f9234207a14c1527eed0335cfaeb8067e04dfa2dfc81110452a2c0503a', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 198, 'bd-2kx', '925200b8025ce0bf7e4dc7d3d19ef702a8fef20cdc97d8c1a4cc639dff83a179', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 199, 'bd-2l6', 'fc16444c3463ee7d58069f79bf5e33e0d0223e79bba33d33b3dcd8c8d4228a7b', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 200, 'bd-2mk', '41efc905a90a119d989f4b0176301d8a7d075e34199b08c6d519a6d6d90301c4', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 201, 'bd-2ng', 'd02177797689f28b08b816917b2adbcc09ac79ef206d37ef5ed34a6cba040e15', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 202, 'bd-2o1', '98b2e22428544fd18aa2e0b4dc1cee6e99403dd8af23d72ada42c07e044a3fe9', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 203, 'bd-2p8', '0560e35fa81d9a509dee4d67c5680665090bbcc62f3b2108a6fa5758089d13e9', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 204, 'bd-2qg', '1fc057db9d993d36cfd6f223cfb6a8d0cfb6699bb29964204e9047948619a39c', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 205, 'bd-2sa', '471778134cb28c79f28c0c9988e23b7acbe58dd96ca20b6ac7a872b9e436f358', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 206, 'bd-2tx', 'e8d8156eb07d89a80af9e87a27a607e27483f3b30b36ff21cc5e4cad14411010', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(262, 262, 3, 207, 'bd-2u3', '44342e53fbf735f0978a40686107bb35cbffaf1916ca6a80f77462ef72a68a82', '2026-03-01T16:02:14.103234464+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 173, 'bd-20k', '72f77028747a275f16e3c51afd7d0760c097164ff5208917bd1e655a94f5e3f8', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 174, 'bd-21j', '5e2cb53d2916c55fdc48dd6b180d4993162f539719e1076803305a0c7efade6f', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 175, 'bd-23z', 'c085111c7e926b552479f872f7cbf7e8fb7e8ed14c3cc60de6d4c1ec08ab4bd4', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 176, 'bd-24a', 'c0216f2acd8e54eac2374604e396266268ab9e92657e76b01cb0eae2a3cd0346', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 177, 'bd-25b', 'b9b5332bbd2dad69426a4b0e90ef00cf0de29f452450b00aa6dbb2ecd2887807', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 178, 'bd-260', '0a78706e3e8a434f4da9a2839028eb7598a1abf7de34e285b606e1b63893f99f', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 179, 'bd-26j', 'c8e8fd782a0c915c5d1003dd00079268d1da3ab198ba97bfb9365dcf6498258e', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 180, 'bd-275', '6d69f1968eae2f46350ebbf6112e93004caa872f2be7eb1fae5329e1f624de5a', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 181, 'bd-27q', '4ba815c310745c6e8609e76b378e371bb5f6418e49b37fd274d2bcd79ac43e15', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 182, 'bd-29g', '295ea1d9d7748918cecd2fbf2b3551b8252cf8b7c949c262f94f34f05e75ce77', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 183, 'bd-2b4', '0cd985a5c3e8d1a855eb91156d7b503aeaef847c5535d050945cb4e952e40386', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 184, 'bd-2b6', 'ed9c5b67e467d1b978d925928d320284c9a3eb0a4bf8509fdb78399ccd66b779', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 185, 'bd-2bh', '0d94138f76efccb4b2e23925e4cddd69bac853c648353163b752438ffc53532b', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 186, 'bd-2cb', '3406639168238754da1a9a766f48b35c7653ee7beafe61b52e55a307aaa586c6', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 187, 'bd-2cg', '0ffae1219bf876d95f991047b87b2ef1b4bad86964142f2571ecc127efdd0074', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 188, 'bd-2cm', '4ead30fa13162734972df934c2012e336a9c13ae7399f2d52e74cace40bd0afd', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 189, 'bd-2df', 'fdf8299b0ed621c9c00e43c4d5aeb704088b98257a7eb3f38882752ca06a567d', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 190, 'bd-2dg', 'beed3b123a4441215970b7708cf42226c1620da78517217891809df606d44c17', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 191, 'bd-2gl', '8016b819a0f1e8ab55b2f931091275f1acc1975fb07f009a206edc9ede40bf6f', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 192, 'bd-2hz', 'ad9e59d8844aa2086a5bddfbe95a087f62005c336dc6c8b6b7d305eb9c76171c', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 193, 'bd-2ii', '72c4a5ee7b47ec63873234221dd3a21ec4550e1000d04177b2c4cf7c97e7213c', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 194, 'bd-2ik', '7c0a32c088dd8d63efca91d6f51052db2f3af846d39684f5ff43dd743f68ad1f', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 195, 'bd-2jh', 'afafe4ac9c6d3b84bd4af572ce0f1afb01d4a1a8e9a85de233d6b891d9cbf0a0', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 196, 'bd-2jv', '83d40916ee7c712da821ffdb02688527d387a235a993bce0a659b0ed44bd00d5', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 197, 'bd-2kq', 'f800b7f9234207a14c1527eed0335cfaeb8067e04dfa2dfc81110452a2c0503a', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 198, 'bd-2kx', '925200b8025ce0bf7e4dc7d3d19ef702a8fef20cdc97d8c1a4cc639dff83a179', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 199, 'bd-2l6', 'fc16444c3463ee7d58069f79bf5e33e0d0223e79bba33d33b3dcd8c8d4228a7b', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 200, 'bd-2mk', '2d178c9b2b3b38cf89d5cf6d6d3e8e7378607f8907478ec16f657e9a08f7eee3', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 201, 'bd-2ng', 'd02177797689f28b08b816917b2adbcc09ac79ef206d37ef5ed34a6cba040e15', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 202, 'bd-2o1', '98b2e22428544fd18aa2e0b4dc1cee6e99403dd8af23d72ada42c07e044a3fe9', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 203, 'bd-2p8', '0560e35fa81d9a509dee4d67c5680665090bbcc62f3b2108a6fa5758089d13e9', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 204, 'bd-2qg', '1fc057db9d993d36cfd6f223cfb6a8d0cfb6699bb29964204e9047948619a39c', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 205, 'bd-2sa', '471778134cb28c79f28c0c9988e23b7acbe58dd96ca20b6ac7a872b9e436f358', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 206, 'bd-2tx', 'e8d8156eb07d89a80af9e87a27a607e27483f3b30b36ff21cc5e4cad14411010', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(263, 263, 3, 207, 'bd-2u3', '44342e53fbf735f0978a40686107bb35cbffaf1916ca6a80f77462ef72a68a82', '2026-03-01T16:12:54.855137896+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1573, 'bd-2ii', '72c4a5ee7b47ec63873234221dd3a21ec4550e1000d04177b2c4cf7c97e7213c', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1574, 'bd-2ik', '7c0a32c088dd8d63efca91d6f51052db2f3af846d39684f5ff43dd743f68ad1f', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1575, 'bd-2jh', 'afafe4ac9c6d3b84bd4af572ce0f1afb01d4a1a8e9a85de233d6b891d9cbf0a0', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1576, 'bd-2jv', '83d40916ee7c712da821ffdb02688527d387a235a993bce0a659b0ed44bd00d5', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1577, 'bd-2kq', 'f800b7f9234207a14c1527eed0335cfaeb8067e04dfa2dfc81110452a2c0503a', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1578, 'bd-2kx', '925200b8025ce0bf7e4dc7d3d19ef702a8fef20cdc97d8c1a4cc639dff83a179', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1579, 'bd-2l6', 'fc16444c3463ee7d58069f79bf5e33e0d0223e79bba33d33b3dcd8c8d4228a7b', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1580, 'bd-2mk', '2d178c9b2b3b38cf89d5cf6d6d3e8e7378607f8907478ec16f657e9a08f7eee3', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1581, 'bd-2ng', 'd02177797689f28b08b816917b2adbcc09ac79ef206d37ef5ed34a6cba040e15', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1582, 'bd-2o1', '98b2e22428544fd18aa2e0b4dc1cee6e99403dd8af23d72ada42c07e044a3fe9', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1583, 'bd-2p8', '0560e35fa81d9a509dee4d67c5680665090bbcc62f3b2108a6fa5758089d13e9', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1584, 'bd-2qg', '1fc057db9d993d36cfd6f223cfb6a8d0cfb6699bb29964204e9047948619a39c', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1585, 'bd-2sa', '471778134cb28c79f28c0c9988e23b7acbe58dd96ca20b6ac7a872b9e436f358', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1586, 'bd-2tx', 'e8d8156eb07d89a80af9e87a27a607e27483f3b30b36ff21cc5e4cad14411010', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1587, 'bd-2u3', '44342e53fbf735f0978a40686107bb35cbffaf1916ca6a80f77462ef72a68a82', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1588, 'bd-2uy', 'b8667fd72f62a2ec28dcb4bee8cb709a85bdf879f590c2ff9c9f7a52e6fd971a', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1589, 'bd-2v0', '8aaf4a1067eee4ae6d3f41619584dfa04c51ebfa97cf981092461a9711e6077e', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1590, 'bd-2vf', '4c0a4288e805e4ef8c1dd8f439b39c0fc3f74c7d86aa3ee084937912c1b2f87b', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1591, 'bd-2w1', '4ef2eae4aeb7665317cc0178583d53004299591708fd8a3d349d7b465d6effd5', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1592, 'bd-2x8', '64cb41944c6ae9fcaf3f600596be0f7c201f0381c4d15c60ad540dc4b2dd6e29', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1593, 'bd-2xa', '70bc971b410d3aa9c4c8da6a02479e35aafa8636268e34144c76a0fe4696c734', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1594, 'bd-2xt', '4320af136f6ad5b9d9a6401c37d20eebb12e3c2bd04f238c12fb839838f5e38d', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1595, 'bd-30o', '2674c2fb16c9f420d56b2b54d8a35e6802b38d20169286cb041538aed180ad5c', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1596, 'bd-318', '47ad1fbabfb6dda55e5e8938ab746107ccb69eeb102d27cb93b116ed528cb512', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1597, 'bd-31x', '3f764ec12e58b58ba5b9d02b44681ae3f3914ad0927b92be3510208abe94fe9c', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1598, 'bd-320', '2683188c4e02f0d36c11f75945f91740ac5d15761799b69ce18af081344fa3e1', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1599, 'bd-321', 'fd9cae6d2203605006e6e8ae4086f62ebcfe0b8eba64885aa7a554dbadcdbe54', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1600, 'bd-338', '4c0cb745450164e091505c1e241bd57c472718ac1bc1711f2166269e418b5015', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1601, 'bd-33b', '6004595a764ae3d01b565fc0f5e649719944faad670cf6e28507d514feb89aaa', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1602, 'bd-345', 'b3cfbc678ccf5d6e3db33e6850362bd2bb2acf8160c962ba986062081c13cf73', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1603, 'bd-34b', 'bd13ed2aeccfb7f934edcac641bd7646e923bd9d8fc1825c2f0ec14e5516395a', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1604, 'bd-34q', '3046c0fadd2c497bb2cc08190e540dc9237a377cc15c8cdee8d4cec4500aae7c', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1605, 'bd-354', 'dcab80773127a2b79ca75f799f53721d3688e220f33352b40a35f9dc21ac9853', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1606, 'bd-37e', '4f8d982942557343b3ee943a68a21848d07fc4ffd09178c41c5f0897cc1dc20d', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(264, 264, 3, 1607, 'bd-37x', 'eea09f0d9955518e07bc6b5ca01388e070853c80e086db10d923e82c909dde1f', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1608, 'bd-387', '96607dde03645139a8127e2021286b4664ffe3ff49de8a4955f639308c30da65', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1609, 'bd-38a', '94177d8c469fe0b0bf9fa41aff1bee5d9d42cef072f7528ec314bcdfa4db0b5a', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1610, 'bd-39r', '83cf52bef8805dc138527e8bc20c4f6fcf0bdc6c2cd56cda96eb725bdb34ad51', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1611, 'bd-3dc', '13dfbeeaa45dd18460ab0d5f7f68e63de7e47a2424da1a417cd935ffaf82d915', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1612, 'bd-3ex', '5fb7a1b7d5dd5dc7617a0c1133060fc78720daacba7bd46bbf8948dd7f80ca97', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1613, 'bd-3ic', 'b4ed9bc0d9f1e017c31a2eca652c25d3499eac751b79ddcdad5a505980869fe7', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1614, 'bd-3if', '9fa79b28fc6d29510cdf5ca1eade29fba39fc42aed9719f51a6ca9d24de81b8b', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1615, 'bd-3lk', '5d0c4639e43e2e4514febeb0929f581f27fbdddff0a72e869654bd5a47672fdd', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(269, 269, 3, 1616, 'bd-3pt', '490cd2c821626a92dd130d7e483b6b5725b91043e8e4de474eea54033b7ad686', '2026-03-01T19:10:39.068440310+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 978, 'bd-81l', 'a19190b7e817792a5fb84ce2b0a9e8b6f8a85179a524b09721a3852cd0d4d326', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 979, 'bd-8sr', '169236d7bba786a2b16de2facd04506e1db107cab963c73ab8d6ef56f2572efc', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 980, 'bd-a11', 'd8e6c324b29a4029722ecbbc8f4bdc3db53f759987eacb3ed87b6de5a26da70b', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 981, 'bd-ahf', 'fdb2d8a98822377ee4ce2c807ae13a302cd195e09500b7f74b1e4a934d979b5f', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 982, 'bd-due', 'dc54a28dda78665d50bcfffa29372ff6cdb727a69c6c2bfc3498c42299854e88', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 983, 'bd-ja2', 'c97980bf04def34d077876d756a160a988d715606efa079590cbfca34df52e16', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 984, 'bd-jqu', '59014799d9bb5cead30e64c4af5c3a8594af4c70149ef1dff7b07a38c5a7a3b9', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 985, 'bd-l79', '4d93e62498c062366e49124ec67d268b56e0afa07a6e754626b4101a061abd1a', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 986, 'bd-lgh', 'ae1f5fdce1b33daa81f9d7f1022bb5f8f6016c1cad193138bbcc8e585245108f', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 987, 'bd-lq3', 'e785dfa01e4c64b2330694ee937af9e8e37e5f4f7b55fc15ae2bdcb0bd5cf6cd', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 988, 'bd-mtu', 'd8b750ac4429327e2863d62090512c2389569b4ccf49a78607c502ca985a9211', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 989, 'bd-n05', 'e23ba26fc93830a48c2ac311747d9d7e9cd462f562d4f198376490eaacbbb346', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 990, 'bd-n75', '1f29e59b46e24d1ad1a48804daa8a17fa83945f68e6fe8c61dae20254480b5e6', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 991, 'bd-nbm', '01cdb7889086873a44155a7adc0b5c82dbcdf0e0c505e4ae9f70b0267cf9833c', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 992, 'bd-o6p', '7ae8ebed9507fc97b9c4ade53be4eb89bf6a9076ca04d4d5c1127a217cbaa89a', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 993, 'bd-ohy', '1704cdc80b5ae63b26d11ae1d73388bfcfde4630bc6f83357d677abe3bb85680', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 994, 'bd-pmf', '97d52b73a3f70bc8e7089328a15706f2ab573469df7a29ece3af9bfdf2065df0', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 995, 'bd-q2i', 'ab3cbdb193fde32501a7e5c858b24e579a41e7dc0c06c288d1df5b15de930f81', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 996, 'bd-rg7', '23f5c180367975e0354e7475a5021bcdbb8fe1aef97c7452f1b3e474c741a506', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 997, 'bd-rn3', '08e27ca42287e478acb20cf6ebf41b1e793d792669d17fb6f45ca69a3c7b81f7', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 998, 'bd-rpv', '692c336504b60f569183cc544aa4a32aa930bf3079f21f1566011bfaf2390e60', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 999, 'bd-s62', 'fb020749d77aaf5f65f67b59ebea967de5191aef8ff1e7cc9740efafe6b33a9e', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1000, 'bd-sa6', '53a40bf88ab731697a365763c6fac77b3a021a1c8c16665c6da3e7e4a2665ef7', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1001, 'bd-tet', 'c0983032a17d2b1a0ef19db33be4804bff56d2021edf641d99ac4925d98d2594', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1002, 'bd-uem', '7a2f1fb78d9e55d338a705924af41a0be74f72c8d1e14373df1acbb50788455d', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1003, 'bd-v3y', '59c47adf299d3838e022890c7cc660db3dd54f4ec7c0a28dd56eaa4c8bba1c45', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1004, 'bd-vh4', 'e8757cb86f427f0914e7292824cf066ce960be8d2ad13d1f0ca7bd3fa4c19886', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1005, 'bd-x3x', '2e5bc44519412d78d3d3cff26d6bbd005dd7ef223c2f20b303f7b694bb58e836', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1006, 'bd-yf9', '19b274261c62bd5f4a74a224790a46b639848d67608a5b88454dfe84938204a0', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1007, 'bd-104', '9b8e6ed19d88daf8fef00a07169c7aa23500c873dd9f882130ee7a3737b4035c', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1008, 'bd-10k', '63b23faff9c9e342aeae314dbb8c73bab592a4dede604f41e889f32deec96f85', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1009, 'bd-11b', '628b159300633c5377f63dc11f2b89fc99733502be97da4d056686dac864aa5e', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1010, 'bd-11c', '42349572692a12009dba8ef127b1951349e64f7b1f99a5f74994393212c6f12c', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1011, 'bd-12b', '8585f7a84d2ee78ca82d03ac34c1c3a346db72baad1d6a39a3cc84349c4c3fec', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(274, 274, 3, 1012, 'bd-163', '207eb0bb10770540eb9481d7f8e8a7d8db4e8a5331c87cc1009d647f29b98771', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 908, 'bd-260', '0a78706e3e8a434f4da9a2839028eb7598a1abf7de34e285b606e1b63893f99f', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 909, 'bd-26j', 'c8e8fd782a0c915c5d1003dd00079268d1da3ab198ba97bfb9365dcf6498258e', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 910, 'bd-275', '6d69f1968eae2f46350ebbf6112e93004caa872f2be7eb1fae5329e1f624de5a', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 911, 'bd-27q', '4ba815c310745c6e8609e76b378e371bb5f6418e49b37fd274d2bcd79ac43e15', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 912, 'bd-29g', '295ea1d9d7748918cecd2fbf2b3551b8252cf8b7c949c262f94f34f05e75ce77', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 913, 'bd-2b4', '0cd985a5c3e8d1a855eb91156d7b503aeaef847c5535d050945cb4e952e40386', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 914, 'bd-2b6', 'ed9c5b67e467d1b978d925928d320284c9a3eb0a4bf8509fdb78399ccd66b779', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 915, 'bd-2bh', '0d94138f76efccb4b2e23925e4cddd69bac853c648353163b752438ffc53532b', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 916, 'bd-2cb', '3406639168238754da1a9a766f48b35c7653ee7beafe61b52e55a307aaa586c6', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 917, 'bd-2cg', '0ffae1219bf876d95f991047b87b2ef1b4bad86964142f2571ecc127efdd0074', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 918, 'bd-2cm', '4ead30fa13162734972df934c2012e336a9c13ae7399f2d52e74cace40bd0afd', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 919, 'bd-2df', 'fdf8299b0ed621c9c00e43c4d5aeb704088b98257a7eb3f38882752ca06a567d', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 920, 'bd-2dg', 'beed3b123a4441215970b7708cf42226c1620da78517217891809df606d44c17', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 921, 'bd-2gl', '8016b819a0f1e8ab55b2f931091275f1acc1975fb07f009a206edc9ede40bf6f', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 922, 'bd-2hz', 'ad9e59d8844aa2086a5bddfbe95a087f62005c336dc6c8b6b7d305eb9c76171c', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 923, 'bd-2ii', '72c4a5ee7b47ec63873234221dd3a21ec4550e1000d04177b2c4cf7c97e7213c', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 924, 'bd-2ik', '7c0a32c088dd8d63efca91d6f51052db2f3af846d39684f5ff43dd743f68ad1f', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 925, 'bd-2jh', 'afafe4ac9c6d3b84bd4af572ce0f1afb01d4a1a8e9a85de233d6b891d9cbf0a0', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 926, 'bd-2jv', '83d40916ee7c712da821ffdb02688527d387a235a993bce0a659b0ed44bd00d5', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 927, 'bd-2kq', 'f800b7f9234207a14c1527eed0335cfaeb8067e04dfa2dfc81110452a2c0503a', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 928, 'bd-2kx', '925200b8025ce0bf7e4dc7d3d19ef702a8fef20cdc97d8c1a4cc639dff83a179', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 929, 'bd-2l6', 'fc16444c3463ee7d58069f79bf5e33e0d0223e79bba33d33b3dcd8c8d4228a7b', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 930, 'bd-2mk', '2d178c9b2b3b38cf89d5cf6d6d3e8e7378607f8907478ec16f657e9a08f7eee3', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 931, 'bd-2ng', 'd02177797689f28b08b816917b2adbcc09ac79ef206d37ef5ed34a6cba040e15', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 932, 'bd-2o1', '98b2e22428544fd18aa2e0b4dc1cee6e99403dd8af23d72ada42c07e044a3fe9', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 933, 'bd-2p8', '0560e35fa81d9a509dee4d67c5680665090bbcc62f3b2108a6fa5758089d13e9', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 934, 'bd-2qg', '1fc057db9d993d36cfd6f223cfb6a8d0cfb6699bb29964204e9047948619a39c', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 935, 'bd-2sa', '471778134cb28c79f28c0c9988e23b7acbe58dd96ca20b6ac7a872b9e436f358', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 936, 'bd-2tx', 'e8d8156eb07d89a80af9e87a27a607e27483f3b30b36ff21cc5e4cad14411010', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 937, 'bd-2u3', '44342e53fbf735f0978a40686107bb35cbffaf1916ca6a80f77462ef72a68a82', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 938, 'bd-2uy', 'b8667fd72f62a2ec28dcb4bee8cb709a85bdf879f590c2ff9c9f7a52e6fd971a', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 939, 'bd-2v0', '8aaf4a1067eee4ae6d3f41619584dfa04c51ebfa97cf981092461a9711e6077e', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 940, 'bd-2vf', '4c0a4288e805e4ef8c1dd8f439b39c0fc3f74c7d86aa3ee084937912c1b2f87b', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 941, 'bd-2w1', '4ef2eae4aeb7665317cc0178583d53004299591708fd8a3d349d7b465d6effd5', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(295, 295, 3, 942, 'bd-2x8', '64cb41944c6ae9fcaf3f600596be0f7c201f0381c4d15c60ad540dc4b2dd6e29', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1013, 'bd-17y', 'c12cc5283d9ed912b07d0e04e93f131676dd882a422aab5f2d58d6fece3d91f6', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1014, 'bd-189', 'c99a930ac23b38aacd85af4a734d715800417975e580f07b8a190985cb953f4c', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1015, 'bd-19h', 'b2d5ffe24038d0e4b8cf0bb6847b259648a8e095a4b53c49170a11a1af980b0d', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1016, 'bd-19t', '384ff04b58e24803323f69eba8c423e0f95bcdf6a97ec6d134e52911af226b21', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1017, 'bd-1ag', 'e9212d655c4c50f373570cc002d801d7ec50e02537f3f3131f3fbbe2f583984e', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1018, 'bd-1db', 'b05838cceeb77572a4dc0d7f32a44c2e1a29a21c16461d66508db8bc943f3499', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1019, 'bd-1et', '8b92d0a7943ba0e737b5f98f2a79495348e09dd02399be7e7461002598e2fa23', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1020, 'bd-1fc', '485e5a4fc44e14334169b5505b3d113809a149b1a011b253746a17ff0bf3e908', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1021, 'bd-1gl', 'ac932eb6ba0cbb03e884771c168426c745a4fce3d1f56c4da3d1324840f68d36', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1022, 'bd-1ik', 'd57ec29174d6e602e15cfb720171573fe2db77eb4062feb66d61d408556bf158', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1023, 'bd-1jm', 'e5878a1ebd011311a031afe381cbd53761d3d7ec2eff49b51c83e1f97840ca47', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1024, 'bd-1kc', 'ca4f1f22b2512cc173479942ff953f469b186d0eee8c2a1dccd41b65706acc9c', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1025, 'bd-1lj', 'bfe1a199024c84aa10d8951dba914911e811465a4bba70b491d9423a83bcea9d', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1026, 'bd-1nb', '9741012571863a48c98616d14e6fdf6d32ebdf0af97bb4efe1cff655a27d496a', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1027, 'bd-1qt', 'e99b9edc3d383d1b35c35c7058d568c5b2d17cecd50b82df9514ce8ff2a5672f', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1028, 'bd-1rv', '8c88dbcac3d9fbbe0125337089a4c6e580594508e42915dcabd3ac06065e4040', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1029, 'bd-1s3', '7bd64088e881b9cef3107c384dd977b2276396721f988bd433efb8cb8aab1682', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1030, 'bd-1td', 'c5c10cefdc7060035e9bffd56840adef8c0f92ac0f8370cc6c89e63691911f47', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1031, 'bd-1u1', 'e33792413459ade9420464e21c80d08e1a5e26b98b8210b7a2f3d6f2e4e48979', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1032, 'bd-1u4', '350944f90e9b7ce039017faf5f27fc4f76e761d9a9c9c6e4f1c42e0a1c54b258', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1033, 'bd-1ua', '23de4461b4c1b284658b6cf4792c236d63771e62117b471d01644382702f49f0', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(302, 302, 3, 1034, 'bd-1ub', '8b1e3849088a477181488335abdc99368d9d013428f18c0c2cc4c51deb421539', '2026-03-01T19:23:09.051070423+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 943, 'bd-2xa', 'bda57fdad7f99cb96edf8548d9a58b35ddb8ce4698a439c6708ecae7e62bafa7', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 944, 'bd-2xt', '4320af136f6ad5b9d9a6401c37d20eebb12e3c2bd04f238c12fb839838f5e38d', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 945, 'bd-30o', '2674c2fb16c9f420d56b2b54d8a35e6802b38d20169286cb041538aed180ad5c', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 946, 'bd-318', '47ad1fbabfb6dda55e5e8938ab746107ccb69eeb102d27cb93b116ed528cb512', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 947, 'bd-31x', '3f764ec12e58b58ba5b9d02b44681ae3f3914ad0927b92be3510208abe94fe9c', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 948, 'bd-320', '2683188c4e02f0d36c11f75945f91740ac5d15761799b69ce18af081344fa3e1', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 949, 'bd-321', 'fd9cae6d2203605006e6e8ae4086f62ebcfe0b8eba64885aa7a554dbadcdbe54', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 950, 'bd-338', '4c0cb745450164e091505c1e241bd57c472718ac1bc1711f2166269e418b5015', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 951, 'bd-33b', '6004595a764ae3d01b565fc0f5e649719944faad670cf6e28507d514feb89aaa', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 952, 'bd-345', 'b3cfbc678ccf5d6e3db33e6850362bd2bb2acf8160c962ba986062081c13cf73', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 953, 'bd-34b', 'bd13ed2aeccfb7f934edcac641bd7646e923bd9d8fc1825c2f0ec14e5516395a', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 954, 'bd-34q', '3046c0fadd2c497bb2cc08190e540dc9237a377cc15c8cdee8d4cec4500aae7c', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 955, 'bd-354', 'dcab80773127a2b79ca75f799f53721d3688e220f33352b40a35f9dc21ac9853', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 956, 'bd-37e', '4f8d982942557343b3ee943a68a21848d07fc4ffd09178c41c5f0897cc1dc20d', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 957, 'bd-37x', 'eea09f0d9955518e07bc6b5ca01388e070853c80e086db10d923e82c909dde1f', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 958, 'bd-387', '96607dde03645139a8127e2021286b4664ffe3ff49de8a4955f639308c30da65', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 959, 'bd-38a', '94177d8c469fe0b0bf9fa41aff1bee5d9d42cef072f7528ec314bcdfa4db0b5a', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 960, 'bd-39r', '83cf52bef8805dc138527e8bc20c4f6fcf0bdc6c2cd56cda96eb725bdb34ad51', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 961, 'bd-3dc', '13dfbeeaa45dd18460ab0d5f7f68e63de7e47a2424da1a417cd935ffaf82d915', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 962, 'bd-3ex', '5fb7a1b7d5dd5dc7617a0c1133060fc78720daacba7bd46bbf8948dd7f80ca97', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 963, 'bd-3ic', 'b4ed9bc0d9f1e017c31a2eca652c25d3499eac751b79ddcdad5a505980869fe7', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 964, 'bd-3if', '9fa79b28fc6d29510cdf5ca1eade29fba39fc42aed9719f51a6ca9d24de81b8b', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 965, 'bd-3lk', '5d0c4639e43e2e4514febeb0929f581f27fbdddff0a72e869654bd5a47672fdd', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 966, 'bd-3pt', '490cd2c821626a92dd130d7e483b6b5725b91043e8e4de474eea54033b7ad686', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 967, 'bd-3qj', 'fcb9d392287dac3d231b554bef6d417858791a7b1df7f42ccb0e8d2d2655e728', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 968, 'bd-3rb', '6f932af181d8c0c8747f0b569bb053c6819e5cfbaa1602afed60684c4d099a57', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 969, 'bd-3sm', '969aea17d94630ffea357be03c6431c59142378b30112c2311085b098cbf1362', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 970, 'bd-3t0', '7c37ff58745763b27717e5aad89035ab03a1e4f0106d5f0353667418b9d48a10', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 971, 'bd-3uq', '578916dd8bacdee221ec5e72fe74f064b74bd978b1b7e1e949a15b9c0ab02c9b', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 972, 'bd-3ve', 'e0e8827440427f2df46e7246e157ad0c32b8e2b69110f1d234b4d59a449bc437', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 973, 'bd-3vt', '83aa935b1dde19a39e0b5bf342fea1b8d32790b5335dc5f52435f3d58c7900f8', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 974, 'bd-51b', 'f642244fddf482657fa4d279853ce494a00f9cf6d8a55a2fe5559a33f05ced56', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 975, 'bd-5id', '3b91dbdbdf182c3a9b57153b1f3057656e0c15c43030df2199db8d4e5e562753', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 976, 'bd-5jh', '0c8e0e1c9a6b2aa3d4cfc1b4c6edc25bcac1d8741afbedde515b202cbbdbc12b', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
INSERT INTO lost_and_found VALUES(305, 305, 3, 977, 'bd-7rt', 'f4d24f80475214ff6e0e62fa06b87c59219d60dc58104e17c5bbdcd6289c3f40', '2026-03-01T19:22:13.973568129+00:00', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL);
CREATE INDEX "idx_dependencies_issue" ON "dependencies" ("issue_id");
CREATE INDEX "idx_dependencies_depends_on" ON "dependencies" ("depends_on_id");
CREATE INDEX "idx_dependencies_type" ON "dependencies" ("type");
CREATE INDEX "idx_dependencies_depends_on_type" ON "dependencies" ("depends_on_id", "type");
CREATE INDEX "idx_dependencies_thread" ON "dependencies" ("thread_id");
CREATE INDEX "idx_dependencies_blocking" ON "dependencies" ("depends_on_id", "issue_id");
CREATE INDEX "idx_labels_label" ON "labels" ("label");
CREATE INDEX "idx_labels_issue" ON "labels" ("issue_id");
CREATE INDEX "idx_comments_issue" ON "comments" ("issue_id");
CREATE INDEX "idx_comments_created_at" ON "comments" ("created_at");
CREATE INDEX "idx_events_issue" ON "events" ("issue_id");
CREATE INDEX "idx_events_type" ON "events" ("event_type");
CREATE INDEX "idx_events_created_at" ON "events" ("created_at");
CREATE INDEX "idx_events_actor" ON "events" ("actor");
CREATE INDEX "idx_dirty_issues_marked_at" ON "dirty_issues" ("marked_at");
CREATE INDEX "idx_blocked_cache_blocked_at" ON "blocked_issues_cache" ("blocked_at");
PRAGMA writable_schema = off;
COMMIT;
