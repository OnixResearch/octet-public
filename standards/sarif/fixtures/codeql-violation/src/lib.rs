/// Deliberately violating fixture for the checked CodeQL producer profile.
///
/// The HTTP URL flows to the modeled reqwest request-url sink. The query pack should report the
/// non-HTTPS URL; this fixture is not a claim that CodeQL finds every such path.
pub fn fetch_insecure() {
    let client = reqwest::blocking::Client::new();
    let _ = client.request(reqwest::Method::GET, "http://example.com/data").send();
}
