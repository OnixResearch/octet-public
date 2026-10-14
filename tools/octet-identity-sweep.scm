;; Octet identity sweep helper.
;; Functional core: path eligibility, occurrence classification, replacement planning,
;; line rewriting, and report rendering. Imperative shell: file walking, file reads,
;; optional writes, and stdout.

(define ROOT ".")
(define MODE-DRY-RUN "dry-run")
(define MODE-APPLY "apply")
(define MODE-VERIFY "verify")
(define MODE-COMPATIBILITY-REPORT "compatibility-report")
(define FORMAT-TEXT "text")
(define FORMAT-JSON "json")
(define FORMAT-FLAG "--format")
(define DEFAULT-FORMAT FORMAT-TEXT)
(define ARG-MODE-INDEX 0)
(define ARG-FORMAT-FLAG-INDEX 1)
(define ARG-FORMAT-VALUE-INDEX 2)
(define LEADING-DOT-SLASH "./")
(define PATH-SEPARATOR #\/)
(define TAB "\t")
(define NEWLINE "\n")
(define EMPTY "")
(define TERMS (list "Slotcar" "slotcar"))
(define CATEGORY-REWRITTEN "rewritten")
(define CATEGORY-ALLOWED-COMPATIBILITY "allowed-compatibility")
(define CATEGORY-ALLOWED-ARCHIVE "allowed-archive")
(define CATEGORY-ALLOWED-PHILOSOPHY "allowed-philosophy")
(define CATEGORY-BLOCKED "blocked")
(define CATEGORIES
  (list CATEGORY-REWRITTEN
        CATEGORY-ALLOWED-COMPATIBILITY
        CATEGORY-ALLOWED-ARCHIVE
        CATEGORY-ALLOWED-PHILOSOPHY
        CATEGORY-BLOCKED))
(define COMPATIBILITY-TEST-DEFAULT "cargo test -p cargo-octet octet_config_and_command_paths -- --nocapture")
(define COMPATIBILITY-TEST-STANDARDS "cargo test -p octet-standards octet_identity_surfaces -- --nocapture")
(define COMPATIBILITY-TEST-DOCS "pi steel file=tools/octet-identity-sweep.scm args='[\"verify\",\"--format\",\"text\"]'")
(define COMPATIBILITY-TEST-LOCK "nix flake lock --no-update-lock-file")

(define SKIP-PARTS
  (list ".git" ".jj" ".direnv" ".pi" ".agent" "target" "node_modules" "result" "result-bin"))

(define BINARY-SUFFIXES
  (list ".a" ".bin" ".bmp" ".bz2" ".dylib" ".gif" ".gz" ".ico" ".jpeg" ".jpg"
        ".o" ".pdf" ".png" ".rlib" ".so" ".svgz" ".tar" ".wasm" ".webp" ".xz" ".zip"))

(define ACTIVE-CHANGE-PREFIX "openspec/changes/mechanical-slotcar-to-octet-sweep/")
(define ACTIVE-CHANGE-EVIDENCE-PREFIX "openspec/changes/mechanical-slotcar-to-octet-sweep/evidence/")
(define ACTIVE-CHANGE-SCRIPT "tools/octet-identity-sweep.scm")
(define LEGACY-PYTHON-CLASSIFIER "tools/classify-slotcar-rename.py")
(define CHANGELOG-PATH "CHANGELOG.md")
(define FLAKE-NIX-PATH "flake.nix")
(define FLAKE-LOCK-PATH "flake.lock")
(define AGENTS-PATH "AGENTS.md")
(define CARGO-OCTET-MANIFEST-PATH "cargo-octet/Cargo.toml")
(define OCTET-STANDARDS-MANIFEST-PATH "crates/octet-standards/Cargo.toml")
(define RELEASE-CANDIDATE-DOC-PATH "docs/release-candidate.md")

(define COMPATIBILITY-PATHS
  (list
   ".agents/skills/slotcar-repo/SKILL.md"
   ACTIVE-CHANGE-SCRIPT
   LEGACY-PYTHON-CLASSIFIER
   FLAKE-NIX-PATH
   CARGO-OCTET-MANIFEST-PATH
   OCTET-STANDARDS-MANIFEST-PATH
   RELEASE-CANDIDATE-DOC-PATH
   "cargo-octet/src/cli.rs"
   "cargo-octet/src/config.rs"
   "cargo-octet/src/fingerprint.rs"
   "cargo-octet/src/provider_manifest.rs"
   "cargo-octet/src/roadmap.rs"
   "cargo-octet/src/transcript.rs"
   "cargo-octet/tests/unison_roadmap.rs"
   "src/config.rs"))

(define STANDARDS-COMPATIBILITY-PATH-SUFFIXES
  (list "src/suppression.rs" "tests/advisory_cli.rs"))

(define COMPATIBILITY-TOKENS
  (list
   "alias"
   "aliases"
   "archive"
   "archived"
   "bootstrap compatibility"
   "compatibility"
   "compatibility alias"
   "compatible"
   "fallback"
   "fallbacks"
   "historical"
   "history"
   "legacy"
   "lower precedence"
   "migration"
   "obsolete"
   "old command"
   "previous command"
   "retained"
   "retire"
   "retired"
   "route to the same engine"
   "source compatibility"
   "source-compatibility"
   "transition"))

(define CODE-COMPATIBILITY-TOKENS
  (list
   "slotcar_binary_identity"
   "workspace.metadata.slotcar"
   "[workspace.metadata.slotcar]"
   "[slotcar]"
   "slotcar-allow:"
   "slotcar-fingerprint:"
   "cargo-slotcar"
   "cargo slotcar"
   "slotcar-transcript"))

(define PATH-COMPATIBILITY-TOKENS
  (list
   "/path/to/git/slotcar"
   "/path/to/slotcar"
   "path:/path/to/slotcar"
   "github:OnixResearch/slotcar"))

(define SWEEP-CONTRACT-PATHS
  (list
   "openspec/specs/consumer-integration/spec.md"
   "openspec/specs/documentation/spec.md"))

(define SWEEP-CONTRACT-TOKENS
  (list
   "Steel dry-run inventories active"
   "Remaining slotcar terms are explicitly allowed"
   "contain `slotcar` command or package names"
   "surface remains after the sweep"
   "without requiring a `slotcar` spelling"
   "repository contains active files with `slotcar`"))

(define (string-contains-from? text needle start)
  (let* ([needle-len (string-length needle)]
         [text-len (string-length text)])
    (cond
      [(= needle-len 0) start]
      [(> (+ start needle-len) text-len) #false]
      [(string=? (substring text start (+ start needle-len)) needle) start]
      [else (string-contains-from? text needle (+ start 1))])))

(define (string-contains-any? text needles)
  (cond
    [(null? needles) #false]
    [(string-contains? text (car needles)) #true]
    [else (string-contains-any? text (cdr needles))]))

(define (string-starts-with? prefix text)
  (pi-string-prefix? prefix text))

(define (string-ends-with? suffix text)
  (pi-string-suffix? suffix text))

(define (lower text) (string-downcase text))

(define (normalize-path path)
  (if (string-starts-with? LEADING-DOT-SLASH path)
      (substring path (string-length LEADING-DOT-SLASH) (string-length path))
      path))

(define (path-parts rel) (pi-string-split-char rel PATH-SEPARATOR))

(define (path-has-skip-part? rel)
  (let loop ([parts (path-parts rel)])
    (cond
      [(null? parts) #false]
      [(pi-member? (car parts) SKIP-PARTS) #true]
      [else (loop (cdr parts))])))

(define (path-has-binary-suffix? rel)
  (let loop ([suffixes BINARY-SUFFIXES])
    (cond
      [(null? suffixes) #false]
      [(string-ends-with? (car suffixes) (lower rel)) #true]
      [else (loop (cdr suffixes))])))

(define (text-file-path? rel)
  (and (not (path-has-skip-part? rel))
       (not (path-has-binary-suffix? rel))))

(define (archive-path? rel)
  (or (string=? rel "openspec/changes/archive")
      (string=? rel "cairn/archive")
      (string-starts-with? "openspec/changes/archive/" rel)
      (string-starts-with? "cairn/archive/" rel)
      (string-contains? rel "/archive/")))

(define (evidence-path? rel)
  (or (string=? rel "openspec/changes/mechanical-slotcar-to-octet-sweep/evidence")
      (string-starts-with? ACTIVE-CHANGE-EVIDENCE-PREFIX rel)
      (and (string-starts-with? "openspec/changes/" rel) (string-contains? rel "/evidence/"))
      (and (string-starts-with? "cairn/changes/" rel) (string-contains? rel "/evidence/"))))

(define (active-change-path? rel)
  (string-starts-with? ACTIVE-CHANGE-PREFIX rel))

(define (standards-crate-path? rel)
  (string-starts-with? "crates/octet-standards/" rel))

(define (standards-compatibility-path? rel)
  (and (standards-crate-path? rel)
       (let loop ([suffixes STANDARDS-COMPATIBILITY-PATH-SUFFIXES])
         (cond
           [(null? suffixes) #false]
           [(string-ends-with? (car suffixes) rel) #true]
           [else (loop (cdr suffixes))]))))

(define (compatibility-path? rel)
  (or (pi-member? rel COMPATIBILITY-PATHS)
      (string-starts-with? ".agents/skills/" rel)
      (string=? rel AGENTS-PATH)
      (string=? rel FLAKE-LOCK-PATH)
      (string=? rel CHANGELOG-PATH)
      (active-change-path? rel)
      (standards-compatibility-path? rel)))

(define (line-has-compatibility-language? line)
  (string-contains-any? (lower line) COMPATIBILITY-TOKENS))

(define (line-has-code-compatibility-token? line)
  (string-contains-any? line CODE-COMPATIBILITY-TOKENS))

(define (line-has-path-compatibility-token? line)
  (string-contains-any? line PATH-COMPATIBILITY-TOKENS))

(define (sweep-contract-path? rel)
  (pi-member? rel SWEEP-CONTRACT-PATHS))

(define (line-has-sweep-contract-token? line)
  (string-contains-any? line SWEEP-CONTRACT-TOKENS))

(define (replacement-for term)
  (if (string=? term "Slotcar") "Octet" "octet"))

(define (focused-test-for rel line)
  (cond
    [(string=? rel FLAKE-LOCK-PATH) COMPATIBILITY-TEST-LOCK]
    [(or (standards-crate-path? rel) (string-contains? line "slotcar-allow:")) COMPATIBILITY-TEST-STANDARDS]
    [(or (string-starts-with? "docs/" rel)
         (string=? rel "README.md")
         (string=? rel "templates/dylint.toml")) COMPATIBILITY-TEST-DOCS]
    [else COMPATIBILITY-TEST-DEFAULT]))

(define (classify-occurrence rel line term)
  (cond
    [(archive-path? rel)
     (hash "category" CATEGORY-ALLOWED-ARCHIVE
           "replacement" EMPTY
           "reason" "historical archive path is scanned but never rewritten"
           "focused_test" "archive rows are asserted by steel-final-verify.txt")]
    [(evidence-path? rel)
     (hash "category" CATEGORY-ALLOWED-ARCHIVE
           "replacement" EMPTY
           "reason" "deterministic evidence artifact is scan-only to avoid recursive proof churn"
           "focused_test" "archive/evidence rows are asserted by steel-final-verify.txt")]
    [(string=? rel ACTIVE-CHANGE-SCRIPT)
     (hash "category" CATEGORY-ALLOWED-COMPATIBILITY
           "replacement" EMPTY
           "reason" "sweep helper must spell the legacy input token it scans and rewrites"
           "focused_test" COMPATIBILITY-TEST-DOCS)]
    [(string=? rel LEGACY-PYTHON-CLASSIFIER)
     (hash "category" CATEGORY-ALLOWED-COMPATIBILITY
           "replacement" EMPTY
           "reason" "legacy classifier retained as a compatibility comparison tool during this sweep"
           "focused_test" COMPATIBILITY-TEST-DOCS)]
    [(compatibility-path? rel)
     (hash "category" CATEGORY-ALLOWED-COMPATIBILITY
           "replacement" EMPTY
           "reason" "path is an explicit compatibility, local-instruction, lock, changelog, or active-change boundary"
           "focused_test" (focused-test-for rel line))]
    [(or (line-has-compatibility-language? line)
         (and (compatibility-path? rel)
              (or (line-has-code-compatibility-token? line)
                  (line-has-path-compatibility-token? line))))
     (hash "category" CATEGORY-ALLOWED-COMPATIBILITY
           "replacement" EMPTY
           "reason" "line explicitly describes retained legacy compatibility or a literal compatibility token"
           "focused_test" (focused-test-for rel line))]
    [(and (sweep-contract-path? rel)
          (line-has-sweep-contract-token? line))
     (hash "category" CATEGORY-ALLOWED-COMPATIBILITY
           "replacement" EMPTY
           "reason" "main spec defines legacy input terms for the Octet identity sweep"
           "focused_test" COMPATIBILITY-TEST-DOCS)]
    [else
     (hash "category" CATEGORY-REWRITTEN
           "replacement" (replacement-for term)
           "reason" "active product identity mechanically rewrites to Octet"
           "focused_test" EMPTY)]))

(define (line-term-positions line term)
  (if (not (string-contains? line term))
      '()
      (let* ([term-len (string-length term)]
             [line-len (string-length line)])
        (let loop ([start 0] [out '()])
          (let ([found (string-contains-from? line term start)])
            (if found
                (loop (+ found term-len) (cons found out))
                (reverse out)))))))

(define (text-has-sweep-term? text)
  (string-contains-any? text TERMS))

(define (occurrence-record rel line-number line term column-zero)
  (let* ([classification (classify-occurrence rel line term)]
         [column-one (+ column-zero 1)]
         [category (hash-ref classification "category")]
         [replacement (hash-ref classification "replacement")]
         [reason (hash-ref classification "reason")]
         [focused-test (hash-ref classification "focused_test")])
    (hash "path" rel
          "line" line-number
          "column" column-one
          "match" term
          "replacement" replacement
          "category" category
          "reason" reason
          "focused_test" focused-test
          "text" (pi-string-trim line))))

(define (line-occurrences rel line-number line)
  (let loop-terms ([terms TERMS] [rows '()])
    (if (null? terms)
        (reverse rows)
        (let* ([term (car terms)]
               [positions (line-term-positions line term)]
               [term-rows (map (lambda (column-zero)
                                 (occurrence-record rel line-number line term column-zero))
                               positions)])
          (loop-terms (cdr terms) (append (reverse term-rows) rows))))))

(define (file-occurrences rel text)
  (let loop-lines ([lines (pi-lines text)] [line-number 1] [rows '()])
    (if (null? lines)
        (reverse rows)
        (let loop-row ([line-rows (line-occurrences rel line-number (car lines))] [next-rows rows])
          (if (null? line-rows)
              (loop-lines (cdr lines) (+ line-number 1) next-rows)
              (loop-row (cdr line-rows) (cons (car line-rows) next-rows)))))))

(define (row-less? left right)
  (let ([left-path (hash-ref left "path")]
        [right-path (hash-ref right "path")]
        [left-line (hash-ref left "line")]
        [right-line (hash-ref right "line")]
        [left-column (hash-ref left "column")]
        [right-column (hash-ref right "column")]
        [left-match (hash-ref left "match")]
        [right-match (hash-ref right "match")])
    (cond
      [(string<? left-path right-path) #true]
      [(string<? right-path left-path) #false]
      [(< left-line right-line) #true]
      [(> left-line right-line) #false]
      [(< left-column right-column) #true]
      [(> left-column right-column) #false]
      [else (string<? left-match right-match)])))

(define (skip-directory-path? rel)
  (let ([parts (path-parts rel)])
    (or (archive-path? rel)
        (evidence-path? rel)
        (and (not (null? parts))
             (pi-member? (car (reverse parts)) SKIP-PARTS)))))

(define (walk-eligible-paths directory)
  (let ([paths '()])
    (let walk ([current directory])
      (let ([iter (read-dir-iter current)])
        (let loop ()
          (let ([entry (read-dir-iter-next! iter)])
            (when entry
              (let* ([entry-path (read-dir-entry-path entry)]
                     [rel (normalize-path entry-path)])
                (if (read-dir-entry-is-dir? entry)
                    (unless (skip-directory-path? rel)
                      (walk entry-path))
                    (when (text-file-path? rel)
                      (set! paths (cons rel paths)))))
              (loop))))))
    paths))

(define (eligible-paths root)
  (pi-sort (walk-eligible-paths root) string<?))

(define (active-scan-path? rel)
  (and (not (archive-path? rel))
       (not (evidence-path? rel))))

(define (archive-summary-path? rel)
  (or (archive-path? rel) (evidence-path? rel)))

(define (scan-paths paths)
  (pi-sort
   (let loop-paths ([remaining paths] [rows '()])
     (if (null? remaining)
         (reverse rows)
         (let* ([rel (car remaining)]
                [text (pi-read-file rel)]
                [file-rows (if (text-has-sweep-term? text) (file-occurrences rel text) '())])
           (let loop-row ([remaining-rows file-rows] [next-rows rows])
             (if (null? remaining-rows)
                 (loop-paths (cdr remaining) next-rows)
                 (loop-row (cdr remaining-rows) (cons (car remaining-rows) next-rows)))))))
   row-less?))

(define (archive-summary-record rel text)
  (hash "path" rel
        "line" 0
        "column" 0
        "match" "slotcar|Slotcar"
        "replacement" EMPTY
        "category" CATEGORY-ALLOWED-ARCHIVE
        "reason" "archive/evidence scope is classify-only and never rewritten by apply mode"
        "focused_test" "archive/evidence rows are asserted by steel-final-verify.txt"
        "text" text))

(define (archive-summary-rows)
  (list
   (archive-summary-record "openspec/changes/archive/**" "historical OpenSpec evidence remains scan-only")
   (archive-summary-record "cairn/archive/**" "historical Cairn evidence remains scan-only")
   (archive-summary-record "openspec/changes/*/evidence/**" "change evidence artifacts remain scan-only to avoid recursive proof churn")))

(define (row-category row) (hash-ref row "category"))
(define (row-path row) (hash-ref row "path"))

(define (rows-in-category rows category)
  (filter (lambda (row) (string=? (row-category row) category)) rows))

(define (count-category rows category)
  (length (rows-in-category rows category)))

(define (count-map rows)
  (let loop ([categories CATEGORIES] [out (hash)])
    (if (null? categories)
        out
        (loop (cdr categories) (hash-insert out (car categories) (count-category rows (car categories)))))))

(define (rewritten-paths rows)
  (pi-sort (pi-unique (map row-path (rows-in-category rows CATEGORY-REWRITTEN))) string<?))

(define (row-active-unapproved? row)
  (string=? (row-category row) CATEGORY-BLOCKED))

(define (planned-rewrite? rel line line-number column-zero term)
  (let ([classification (classify-occurrence rel line term)])
    (string=? (hash-ref classification "category") CATEGORY-REWRITTEN)))

(define (rewrite-line rel line-number line)
  (let ([line-len (string-length line)])
    (let loop ([index 0] [parts '()])
      (cond
        [(= index line-len) (apply string-append (reverse parts))]
        [(and (<= (+ index (string-length "Slotcar")) line-len)
              (string=? (substring line index (+ index (string-length "Slotcar"))) "Slotcar"))
         (if (planned-rewrite? rel line line-number index "Slotcar")
             (loop (+ index (string-length "Slotcar")) (cons "Octet" parts))
             (loop (+ index (string-length "Slotcar")) (cons "Slotcar" parts)))]
        [(and (<= (+ index (string-length "slotcar")) line-len)
              (string=? (substring line index (+ index (string-length "slotcar"))) "slotcar"))
         (if (planned-rewrite? rel line line-number index "slotcar")
             (loop (+ index (string-length "slotcar")) (cons "octet" parts))
             (loop (+ index (string-length "slotcar")) (cons "slotcar" parts)))]
        [else (loop (+ index 1) (cons (string (string-ref line index)) parts))]))))

(define (rewrite-text rel text)
  (let loop ([lines (pi-lines text)] [line-number 1] [out '()])
    (if (null? lines)
        (string-join (reverse out) NEWLINE)
        (loop (cdr lines)
              (+ line-number 1)
              (cons (rewrite-line rel line-number (car lines)) out)))))

(define (pi-write-file/truncate path text)
  (delete-file! path)
  (pi-write-file path text))

(define (apply-rewrites paths)
  (let loop ([remaining paths] [modified '()])
    (if (null? remaining)
        (reverse modified)
        (let* ([rel (car remaining)]
               [old-text (pi-read-file rel)]
               [new-text (rewrite-text rel old-text)])
          (if (string=? old-text new-text)
              (loop (cdr remaining) modified)
              (begin
                (pi-write-file/truncate rel new-text)
                (loop (cdr remaining) (cons rel modified))))))))

(define (path-action-row rel)
  (let ([replacement (cond
                       [(string-contains? rel "crates/slotcar-standards")
                        (string-append "rename path to " "crates/octet-standards")]
                       [(string-contains? rel "slotcar-standards")
                        "rename path segment to octet-standards"]
                       [else "review path rename or compatibility allowlist"])])
    (hash "path" rel
          "replacement" replacement
          "reason" "path contains legacy slotcar identity; text apply cannot rename filesystem paths")))

(define (path-actions paths)
  (map path-action-row
       (filter (lambda (rel)
                 (and (string-contains? rel "slotcar")
                      (not (archive-path? rel))
                      (not (evidence-path? rel))
                      (not (compatibility-path? rel))))
               paths)))

(define (report-summary mode paths rows modified-paths)
  (let* ([rewrites (rows-in-category rows CATEGORY-REWRITTEN)]
         [blocked-count (count-category rows CATEGORY-BLOCKED)]
         [counts (count-map rows)]
         [rewrite-paths (rewritten-paths rows)]
         [actions (path-actions paths)])
    (hash "mode" mode
          "root" ROOT
          "files_scanned" (length paths)
          "files_modified" (length modified-paths)
          "occurrences_total" (length rows)
          "rewrite_files_planned" (length rewrite-paths)
          "rewrite_occurrences_planned" (length rewrites)
          "archives_rewritten" 0
          "blocked" blocked-count
          "unapproved_active_slotcar" blocked-count
          "counts" counts
          "modified_paths" modified-paths
          "path_actions" actions)))

(define (row->text row)
  (string-join
   (list (hash-ref row "category")
         (hash-ref row "path")
         (number->string (hash-ref row "line"))
         (number->string (hash-ref row "column"))
         (hash-ref row "match")
         (hash-ref row "replacement")
         (hash-ref row "reason")
         (hash-ref row "focused_test")
         (hash-ref row "text"))
   TAB))

(define (path-action->text row)
  (string-join
   (list (hash-ref row "path")
         (hash-ref row "replacement")
         (hash-ref row "reason"))
   TAB))

(define (render-count-lines counts)
  (map (lambda (category)
         (string-append "count." category ": " (number->string (hash-ref counts category))))
       CATEGORIES))

(define (render-text-report summary rows)
  (let* ([counts (hash-ref summary "counts")]
         [modified-paths (hash-ref summary "modified_paths")]
         [actions (hash-ref summary "path_actions")]
         [header-lines
          (append
           (list
            "Octet identity sweep report"
            (string-append "mode: " (hash-ref summary "mode"))
            (string-append "root: " (hash-ref summary "root"))
            (string-append "files_scanned: " (number->string (hash-ref summary "files_scanned")))
            (string-append "files_modified: " (number->string (hash-ref summary "files_modified")))
            (string-append "occurrences_total: " (number->string (hash-ref summary "occurrences_total")))
            (string-append "rewrite_files_planned: " (number->string (hash-ref summary "rewrite_files_planned")))
            (string-append "rewrite_occurrences_planned: " (number->string (hash-ref summary "rewrite_occurrences_planned")))
            (string-append "archives_rewritten: " (number->string (hash-ref summary "archives_rewritten")))
            (string-append "blocked: " (number->string (hash-ref summary "blocked")))
            (string-append "unapproved_active_slotcar: " (number->string (hash-ref summary "unapproved_active_slotcar"))))
           (render-count-lines counts)
           (list
            "modified_paths:"
            (if (null? modified-paths) "  (none)" (string-join (map (lambda (path) (string-append "  " path)) modified-paths) NEWLINE))
            "path_actions:"
            "columns: path<TAB>replacement<TAB>reason"
            (if (null? actions) "  (none)" (string-join (map path-action->text actions) NEWLINE))
            "occurrences:"
            "columns: category<TAB>path<TAB>line<TAB>column<TAB>match<TAB>replacement<TAB>reason<TAB>focused_test<TAB>text"))])
    (string-join (append header-lines (map row->text rows)) NEWLINE)))

(define (compatibility-row? row)
  (string=? (row-category row) CATEGORY-ALLOWED-COMPATIBILITY))

(define (compatibility-decision-line row)
  (string-join
   (list "retained"
         (hash-ref row "path")
         (number->string (hash-ref row "line"))
         (number->string (hash-ref row "column"))
         (hash-ref row "match")
         (hash-ref row "reason")
         (hash-ref row "focused_test"))
   TAB))

(define (render-compatibility-report summary rows)
  (let* ([compatibility-rows (filter compatibility-row? rows)]
         [lines (append
                 (list
                  "Octet compatibility decision report"
                  (string-append "mode: " MODE-COMPATIBILITY-REPORT)
                  (string-append "retained_compatibility: " (number->string (length compatibility-rows)))
                  (string-append "blocked: " (number->string (hash-ref summary "blocked")))
                  "columns: decision<TAB>path<TAB>line<TAB>column<TAB>match<TAB>rationale<TAB>focused_test")
                 (map compatibility-decision-line compatibility-rows))])
    (string-join lines NEWLINE)))

(define (render-json-report summary rows)
  (pi-json-write/pretty (hash "summary" summary "occurrences" rows)))

(define (render-report mode format summary rows)
  (cond
    [(string=? mode MODE-COMPATIBILITY-REPORT)
     (if (string=? format FORMAT-JSON)
         (render-json-report summary (filter compatibility-row? rows))
         (render-compatibility-report summary rows))]
    [(string=? format FORMAT-JSON) (render-json-report summary rows)]
    [else (render-text-report summary rows)]))

(define (parse-mode)
  (pi-arg-ref ARG-MODE-INDEX MODE-DRY-RUN))

(define (parse-format)
  (if (and (>= (pi-arg-count) (+ ARG-FORMAT-VALUE-INDEX 1))
           (string=? (pi-arg-ref ARG-FORMAT-FLAG-INDEX EMPTY) FORMAT-FLAG))
      (pi-arg-ref ARG-FORMAT-VALUE-INDEX DEFAULT-FORMAT)
      DEFAULT-FORMAT))

(define (validate-mode mode)
  (unless (pi-member? mode (list MODE-DRY-RUN MODE-APPLY MODE-VERIFY MODE-COMPATIBILITY-REPORT))
    (error (string-append "unknown mode: " mode))))

(define (validate-format format)
  (unless (pi-member? format (list FORMAT-TEXT FORMAT-JSON))
    (error (string-append "unknown format: " format))))

(define (main)
  (let* ([mode (parse-mode)]
         [format (parse-format)])
    (validate-mode mode)
    (validate-format format)
    (let* ([paths (eligible-paths ROOT)]
           [active-paths (filter active-scan-path? paths)]
           [rows-before (pi-sort (append (scan-paths active-paths) (archive-summary-rows)) row-less?)]
           [modified-paths (if (string=? mode MODE-APPLY)
                               (apply-rewrites (rewritten-paths rows-before))
                               '())]
           [rows-after (if (string=? mode MODE-APPLY)
                           (pi-sort (append (scan-paths active-paths) (archive-summary-rows)) row-less?)
                           rows-before)]
           [summary (report-summary mode paths rows-after modified-paths)])
      (println (render-report mode format summary rows-after))
      (when (and (string=? mode MODE-VERIFY)
                 (> (hash-ref summary "blocked") 0))
        (error "verify failed: blocked legacy identity rows remain")))))

(main)
