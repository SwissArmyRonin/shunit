#[derive(Clone, Debug, Default, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "property")]
pub struct Property {
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(attribute)]
    pub value: String,
}

#[derive(Clone, Debug, Default, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "properties")]
pub struct Properties {
    #[yaserde(child, rename = "property")]
    pub properties: Vec<Property>,
}

#[derive(Clone, Debug, Default, PartialEq, YaDeserialize, YaSerialize)]
pub struct TestError {
    #[yaserde(attribute)]
    pub message: String,
    #[yaserde(attribute, rename = "type")]
    pub error_type: String,
    #[yaserde(text)]
    pub body: String,
}

#[derive(Clone, Debug, Default, PartialEq, YaDeserialize, YaSerialize)]
pub struct TestCase {
    #[yaserde(attribute)]
    pub classname: String,
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(attribute)]
    pub time: f32,
    #[yaserde(child)]
    pub error: Option<TestError>,
}

#[derive(Clone, Debug, Default, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "testsuite")]
pub struct TestSuite {
    #[yaserde(attribute)]
    pub errors: u32,
    #[yaserde(attribute)]
    pub failures: u32,
    #[yaserde(attribute)]
    pub hostname: String,
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(attribute)]
    pub tests: u32,
    #[yaserde(attribute)]
    pub time: f32,
    #[yaserde(attribute)]
    pub timestamp: String,
    #[yaserde(child)]
    pub properties: Properties,
    #[yaserde(rename = "system-out")]
    pub system_out: String,
    #[yaserde(rename = "system-err")]
    pub system_err: String,
    #[yaserde(rename = "testcase")]
    pub testcases: Vec<TestCase>,
}
