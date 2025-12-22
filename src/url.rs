struct Url {

}

impl Url {
    fn new(url: &str) -> Self {
        // Extract the scheme, which is separated by the URL by ://.
        // Browser currently only supports http so let's check that too.
        let url_split = url.splitn(2, "://").collect::<Vec<_>>();
        let scheme = url_split[0];
        let url = url_split[1];
        assert_eq!(scheme, "http");

        return Url {}
    }
}