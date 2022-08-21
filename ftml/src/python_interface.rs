use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use pyo3::prelude::*;

use crate::data::PageRef;
use crate::includes::{FetchedPage, IncludeRef};
use crate::info::VERSION;
use crate::prelude::*;
use crate::render::html::HtmlRender;
use crate::render::text::TextRender;

fn render<R: Render>(input: &mut String, renderer: &R, page_info: PageInfo, callbacks: Py<PyAny>) -> R::Output
{
    let mut settings = WikitextSettings::from_mode(WikitextMode::Page);
    settings.syntax_compatibility = true;
    settings.use_include_compatibility = true;

    let includer = PythonCallbacks{ callbacks: Box::new(callbacks.clone()) };
    let page_callbacks = Rc::new(PythonCallbacks{ callbacks: Box::new(callbacks) });

    // Substitute page inclusions
    let (mut text, _included_pages) = include(input, &settings, includer, || panic!("Mismatched includer page count")).unwrap_or((input.to_owned(), vec![]));

    preprocess(&mut text);
    let tokens = tokenize(&mut text);
    let (tree, _warnings) = parse(&tokens, &page_info, page_callbacks.clone(), &settings).into();
    let output = renderer.render(&tree, &page_info, page_callbacks, &settings);
    output
}

#[pyclass(name="IncludeRef")]
struct PyIncludeRef {
    #[pyo3(get)]
    pub full_name: String,
    #[pyo3(get)]
    pub variables: HashMap<String, String>
}

impl<'t> From<&IncludeRef<'t>> for PyIncludeRef {
    fn from(r: &IncludeRef<'t>) -> Self {
        let variables = r.variables();
        let py_variables: HashMap<String, String> = variables.keys().fold(HashMap::new(), |mut acc, k| {
            acc.insert(k.to_string(), variables.get(k).unwrap().to_string());
            acc
        });

        Self {
            full_name: r.page_ref().to_string(),
            variables: py_variables
        }
    }
}

#[pyclass(name="FetchedPage")]
struct PyFetchedPage {
    full_name: String,
    content: Option<String>,
}

#[pymethods]
impl PyFetchedPage {
    #[new]
    fn new(
        full_name: String,
        content: Option<String>
    ) -> Self {
        Self { full_name: full_name.clone(), content: content.clone() }
    }
}

impl PyFetchedPage {
    fn to_fetched_page(&self) -> FetchedPage<'static> {
        let content = match &self.content {
            Some(content) => Some(Cow::from(content.to_owned())),
            None => None
        };

        FetchedPage{
            page_ref: PageRef::parse(&self.full_name).expect("PageRef returned back from Python is invalid"),
            content
        }
    }
}

#[pyclass(name="PageInfo")]
struct PyPageInfo {
    page: String,
    category: String,
    site: String,
    title: String,
    domain: String,
    media_domain: String,
    rating: f64,
    tags: Vec<String>,
    language: String
}

#[pymethods]
impl PyPageInfo {
    #[new]
    fn new(
        page: String,
        category: String,
        domain: String,
        media_domain: Option<String>,
        site: Option<String>,
        title: Option<String>,
        rating: Option<f64>,
        tags: Option<Vec<String>>,
        language: Option<String>
    ) -> Self {
        let domain_split: Vec<&str> = domain.split(".").collect();

        Self {
            page: page.clone(),
            category,
            domain: domain.clone(),
            media_domain: media_domain.unwrap_or(domain.clone()),
            site: site.unwrap_or(String::from(*domain_split.first().unwrap())),
            title: title.unwrap_or(page.clone()),
            rating: rating.unwrap_or(0.0),
            tags: tags.unwrap_or(vec![]),
            language: language.unwrap_or(String::from("default")),
        }
    }
}

impl PyPageInfo {
    fn to_page_info(&self) -> PageInfo<'static> {
        PageInfo { 
            page: Cow::Owned(self.page.clone()), 
            category: Some(Cow::Owned(self.category.clone())),
            site: Cow::Owned(self.site.clone()),
            domain: Cow::Owned(self.domain.clone()),
            media_domain: Cow::Owned(self.media_domain.clone()), 
            title: Cow::Owned(self.title.clone()), // why is this part of page info?
            alt_title: None, // why is this part of page info?
            rating: self.rating, 
            tags: self.tags.clone().iter().map(|x| Cow::Owned(x.clone())).collect(),
            language: Cow::Owned(self.language.clone()),
        }
    }
}

struct PythonCallbacks {
    callbacks: Box<Py<PyAny>>,
}

impl Debug for PythonCallbacks {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<PythonCallbacks>")
    }
}

impl PageCallbacks for PythonCallbacks {
    fn module_has_body(&self, module_name: Cow<str>) -> bool {
        let result = Python::with_gil(|py| {
            return self.callbacks.getattr(py, "module_has_body")?.call(py, (module_name,), None)?.extract(py);
        });
        match result {
            Ok(result) => result,
            Err(_) => false
        }
    }

    fn render_module(&self, module_name: Cow<str>, params: HashMap<Cow<str>, Cow<str>>, body: Cow<str>) -> Cow<'static, str> {
        let py_params: HashMap<String, String> = params.keys().fold(HashMap::new(), |mut acc, k| {
            acc.insert(k.to_string(), params.get(k).unwrap().to_string());
            acc
        });
        let result: PyResult<String> = Python::with_gil(|py| {
            return self.callbacks.getattr(py, "render_module")?.call(py, (module_name, py_params, body), None)?.extract(py);
        });
        match result {
            Ok(result) => Cow::from(result.as_str().to_owned()),
            Err(_) => Cow::from("")
        }
    }

    fn render_user(&self, user: Cow<str>, avatar: bool) -> Cow<'static, str> {
        let result: PyResult<String> = Python::with_gil(|py| {
            return self.callbacks.getattr(py, "render_user")?.call(py, (user, avatar), None)?.extract(py);
        });
        match result {
            Ok(result) => Cow::from(result.as_str().to_owned()),
            Err(_) => Cow::from("")
        }
    }

    fn get_i18n_message<'a>(&self, message_id: Cow<str>) -> Cow<'static, str> {
        let result: PyResult<String> = Python::with_gil(|py| {
            return self.callbacks.getattr(py, "get_i18n_message")?.call(py, (message_id,), None)?.extract(py);
        });
        match result {
            Ok(result) => Cow::from(result.as_str().to_owned()),
            Err(_) => Cow::from("?")
        }
    }
}

impl<'t> Includer<'t> for PythonCallbacks {
    type Error = ();

    #[inline]
    fn include_pages(
        &mut self,
        includes: &[IncludeRef<'t>],
    ) -> Result<Vec<FetchedPage<'t>>, ()> {
        let py_includes: Vec<PyIncludeRef> = includes.iter().map(|x| PyIncludeRef::from(x)).collect();
        let result: PyResult<Vec<FetchedPage>> = Python::with_gil(|py| {
            Ok(self.callbacks.getattr(py, "fetch_includes")?.call(py, (py_includes,), None)?.extract::<Vec<PyRef<PyFetchedPage>>>(py)
                ?.iter().map(|x| x.to_fetched_page()).collect())
        });
        match result {
            Ok(fetched_pages) => Ok(fetched_pages),
            Err(_) => Err(())
        }
    }

    #[inline]
    fn no_such_include(&mut self, page_ref: &PageRef<'t>) -> Result<Cow<'t, str>, ()> {
        let result: PyResult<String> = Python::with_gil(|py| {
            return self.callbacks.getattr(py, "render_include_not_found")?.call(py, (page_ref.to_string(),), None)?.extract(py);
        });
        match result {
            Ok(result) => Ok(Cow::from(result.as_str().to_owned())),
            Err(_) => Err(())
        }
    }
}

#[pyclass(subclass)]
struct Callbacks {}

#[pymethods]
impl Callbacks {
    #[new]
    fn new() -> Self {
        Callbacks{}
    }

    pub fn module_has_body(&self, _module_name: String) -> PyResult<bool> {
        return Ok(false)
    }

    pub fn render_module(&self, module_name: String, _params: HashMap<String, String>, _body: String) -> PyResult<String> {
        return Ok(format!("UnimplementedModule[{module_name}]").to_string())
    }

    pub fn render_user(&self, user: String, _avatar: bool) -> PyResult<String> {
        return Ok(format!("UnimplementedUser[{user}]").to_string())
    }

    pub fn get_i18n_message(&self, _message_id: String) -> PyResult<String> {
        return Ok(String::from("?"))
    }

    pub fn render_include_not_found(&self, full_name: String) -> PyResult<String> {
        return Ok(format!("UnimplementedIncludeNotFound[{full_name}]").to_string())
    }

    pub fn fetch_includes(&self, includes: Vec<PyRef<PyIncludeRef>>) -> PyResult<Vec<PyFetchedPage>> {
        return Ok(includes.iter().map(|x| PyFetchedPage{full_name: x.full_name.to_owned(), content: None}).collect())
    }
}

#[pyfunction(kwargs="**")]
fn render_html(source: String, callbacks: Py<PyAny>, page_info: &PyPageInfo) -> PyResult<HashMap<String, String>>
{
    let html_output = render(&mut source.to_string(), &HtmlRender, page_info.to_page_info(), callbacks);

    let mut output = HashMap::new();
    output.insert(String::from("body"), html_output.body);
    output.insert(String::from("style"), html_output.styles.join("\n"));

    Ok(output)
}


#[pyfunction(kwargs="**")]
fn render_text(source: String, callbacks: Py<PyAny>, page_info: &PyPageInfo) -> PyResult<String>
{
    Ok(render(&mut source.to_string(), &TextRender, page_info.to_page_info(), callbacks))
}


#[pymodule]
fn ftml(_py: Python, m: &PyModule) -> PyResult<()> {

    m.add("ftml_version", VERSION.to_string())?;
    m.add_function(wrap_pyfunction!(render_html, m)?)?;
    m.add_function(wrap_pyfunction!(render_text, m)?)?;
    m.add_class::<Callbacks>()?;
    m.add_class::<PyPageInfo>()?;
    m.add_class::<PyIncludeRef>()?;
    m.add_class::<PyFetchedPage>()?;

    Ok(())
}
