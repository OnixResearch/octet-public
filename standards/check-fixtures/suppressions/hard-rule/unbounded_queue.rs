fn fixture() {
    // tigerstyle-allow: policy=tigerstyle.embedded.bounded-queues; source=docs/standards/source/embedded.adoc#bounded-everything; owner=platform; rationale=fixture proves a justified next-line suppression; review=when suppression schema changes; safer_default=bounded queue; scope=next-line
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel();
}

fn unsuppressed() {
    let (_tx, _rx) = tokio::sync::mpsc::unbounded_channel();
}
