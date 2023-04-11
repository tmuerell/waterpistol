use models::report::TestrunData;
use yewdux::store::Store;

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct TestrunDataSelection {
    pub testrun_data: Option<TestrunData>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Store)]
pub struct CompareSelection {
    pub testrun_data: Option<Vec<TestrunData>>,
}