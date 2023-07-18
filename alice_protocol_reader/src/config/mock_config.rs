use super::filter::FilterOpt;

#[derive(Default, Debug)]
pub struct MockConfig {
    pub(crate) filter_link: Option<u8>,
    pub(crate) filter_fee: Option<u16>,
    pub(crate) filter_its_stave: Option<u16>,
    pub(crate) skip_payload: bool,
}

impl FilterOpt for MockConfig {
    fn skip_payload(&self) -> bool {
        self.skip_payload
    }

    fn filter_link(&self) -> Option<u8> {
        self.filter_link
    }

    fn filter_fee(&self) -> Option<u16> {
        self.filter_fee
    }

    fn filter_its_stave(&self) -> Option<u16> {
        self.filter_its_stave
    }
}
