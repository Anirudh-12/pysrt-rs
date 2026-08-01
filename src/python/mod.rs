use pyo3::create_exception;
use pyo3::exceptions::{PyAttributeError, PyIndexError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PySlice, PyString, PyTuple, PyType};

use crate::error::SrtError;
use crate::file::{ErrorHandling, SubRipFile as RustSubRipFile};
use crate::item::{ItemIndex, SubRipItem as RustSubRipItem};
use crate::time::SubRipTime as RustSubRipTime;

create_exception!(pysrt, Error, pyo3::exceptions::PyException);
create_exception!(pysrt, InvalidItem, Error);
create_exception!(pysrt, InvalidTimeString, Error);

fn to_py_err(err: SrtError) -> PyErr {
    match err {
        SrtError::InvalidTimeString(msg) => InvalidTimeString::new_err(msg),
        SrtError::InvalidItem(msg) => InvalidItem::new_err(msg),
        other => InvalidItem::new_err(other.to_string()),
    }
}

fn coerce_time(other: &Bound<'_, PyAny>) -> PyResult<Py<PySubRipTime>> {
    let py = other.py();
    if let Ok(t) = other.extract::<Py<PySubRipTime>>() {
        return Ok(t);
    }
    if let Ok(s) = other.extract::<String>() {
        let inner = RustSubRipTime::from_string(&s).map_err(to_py_err)?;
        return Py::new(py, PySubRipTime { inner });
    }
    if let Ok(ord) = other.extract::<i64>() {
        let inner = RustSubRipTime::from_ordinal(ord);
        return Py::new(py, PySubRipTime { inner });
    }
    if other.hasattr("hour")? && other.hasattr("minute")? {
        let hour: i64 = other.getattr("hour")?.extract()?;
        let minute: i64 = other.getattr("minute")?.extract()?;
        let second: i64 = other.getattr("second")?.extract()?;
        let microsecond: i64 = other.getattr("microsecond")?.extract()?;
        return Py::new(
            py,
            PySubRipTime::new(hour, minute, second, microsecond / 1000),
        );
    }
    if let Ok(dict) = other.downcast::<PyDict>() {
        let mut h = 0;
        let mut m = 0;
        let mut s = 0;
        let mut ms = 0;
        if let Some(v) = dict.get_item("hours")? {
            h = v.extract()?;
        }
        if let Some(v) = dict.get_item("minutes")? {
            m = v.extract()?;
        }
        if let Some(v) = dict.get_item("seconds")? {
            s = v.extract()?;
        }
        if let Some(v) = dict.get_item("milliseconds")? {
            ms = v.extract()?;
        }
        return Py::new(py, PySubRipTime::new(h, m, s, ms));
    }
    if let Ok(tuple) = other.downcast::<PyTuple>() {
        let mut h = 0;
        let mut m = 0;
        let mut s = 0;
        let mut ms = 0;
        if tuple.len() > 0 {
            h = tuple.get_item(0)?.extract()?;
        }
        if tuple.len() > 1 {
            m = tuple.get_item(1)?.extract()?;
        }
        if tuple.len() > 2 {
            s = tuple.get_item(2)?.extract()?;
        }
        if tuple.len() > 3 {
            ms = tuple.get_item(3)?.extract()?;
        }
        return Py::new(py, PySubRipTime::new(h, m, s, ms));
    }
    Err(PyTypeError::new_err("Cannot coerce to SubRipTime"))
}

#[pyclass(name = "_TimeDescriptor", module = "pysrt")]
struct PyTimeDescriptor {
    field: u8,
}

#[pymethods]
impl PyTimeDescriptor {
    fn __get__(
        &self,
        instance: &Bound<'_, PyAny>,
        _owner: &Bound<'_, PyAny>,
    ) -> PyResult<PyObject> {
        let py = instance.py();
        if instance.is_none() {
            return Err(PyAttributeError::new_err("Descriptor accessed on class"));
        }
        let time = instance.extract::<PyRef<PySubRipTime>>()?;
        let val = match self.field {
            0 => time.inner.hours(),
            1 => time.inner.minutes(),
            2 => time.inner.seconds(),
            3 => time.inner.milliseconds(),
            _ => time.inner.ordinal,
        };
        Ok(val.to_object(py))
    }

    fn __set__(
        &self,
        instance: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        if instance.is_none() {
            return Err(PyAttributeError::new_err("Descriptor accessed on class"));
        }
        let mut time = instance.extract::<PyRefMut<PySubRipTime>>()?;
        let val: i64 = value.extract()?;
        match self.field {
            0 => time.inner.set_hours(val),
            1 => time.inner.set_minutes(val),
            2 => time.inner.set_seconds(val),
            3 => time.inner.set_milliseconds(val),
            _ => time.inner.ordinal = val,
        }
        Ok(())
    }
}

#[pyclass(name = "SubRipTime", module = "pysrt")]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PySubRipTime {
    pub inner: RustSubRipTime,
}

#[pymethods]
impl PySubRipTime {
    #[new]
    #[pyo3(signature = (hours=0, minutes=0, seconds=0, milliseconds=0))]
    pub fn new(hours: i64, minutes: i64, seconds: i64, milliseconds: i64) -> Self {
        Self {
            inner: RustSubRipTime::new(hours, minutes, seconds, milliseconds),
        }
    }

    #[getter]
    fn ordinal(&self) -> i64 {
        self.inner.ordinal
    }

    #[setter]
    fn set_ordinal(&mut self, val: i64) {
        self.inner.ordinal = val;
    }

    #[getter]
    fn hours(&self) -> i64 {
        self.inner.hours()
    }

    #[setter]
    fn set_hours(&mut self, val: i64) {
        self.inner.set_hours(val);
    }

    #[getter]
    fn minutes(&self) -> i64 {
        self.inner.minutes()
    }

    #[setter]
    fn set_minutes(&mut self, val: i64) {
        self.inner.set_minutes(val);
    }

    #[getter]
    fn seconds(&self) -> i64 {
        self.inner.seconds()
    }

    #[setter]
    fn set_seconds(&mut self, val: i64) {
        self.inner.set_seconds(val);
    }

    #[getter]
    fn milliseconds(&self) -> i64 {
        self.inner.milliseconds()
    }

    #[setter]
    fn set_milliseconds(&mut self, val: i64) {
        self.inner.set_milliseconds(val);
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "SubRipTime({}, {}, {}, {})",
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds()
        )
    }

    fn __add__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let coerced = coerce_time(other)?;
        let other_inner = coerced.borrow(py).inner;
        Ok(Self {
            inner: self.inner + other_inner,
        })
    }

    fn __iadd__(&mut self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<()> {
        let coerced = coerce_time(other)?;
        self.inner.ordinal += coerced.borrow(py).inner.ordinal;
        Ok(())
    }

    fn __sub__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<Self> {
        let coerced = coerce_time(other)?;
        let other_inner = coerced.borrow(py).inner;
        Ok(Self {
            inner: self.inner - other_inner,
        })
    }

    fn __isub__(&mut self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<()> {
        let coerced = coerce_time(other)?;
        self.inner.ordinal -= coerced.borrow(py).inner.ordinal;
        Ok(())
    }

    fn __mul__(&self, ratio: f64) -> Self {
        Self {
            inner: self.inner * ratio,
        }
    }

    fn __imul__(&mut self, ratio: f64) {
        self.inner *= ratio;
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(coerced) = coerce_time(other) {
            self.inner.ordinal == coerced.borrow(py).inner.ordinal
        } else {
            false
        }
    }

    fn __lt__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let coerced = coerce_time(other)?;
        let ord = coerced.borrow(py).inner.ordinal;
        Ok(self.inner.ordinal < ord)
    }

    fn __le__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let coerced = coerce_time(other)?;
        let ord = coerced.borrow(py).inner.ordinal;
        Ok(self.inner.ordinal <= ord)
    }

    fn __gt__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let coerced = coerce_time(other)?;
        let ord = coerced.borrow(py).inner.ordinal;
        Ok(self.inner.ordinal > ord)
    }

    fn __ge__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let coerced = coerce_time(other)?;
        let ord = coerced.borrow(py).inner.ordinal;
        Ok(self.inner.ordinal >= ord)
    }

    fn __len__(&self) -> usize {
        4
    }

    fn __getitem__(&self, idx: isize) -> PyResult<i64> {
        match idx {
            0 | -4 => Ok(self.hours()),
            1 | -3 => Ok(self.minutes()),
            2 | -2 => Ok(self.seconds()),
            3 | -1 => Ok(self.milliseconds()),
            _ => Err(PyIndexError::new_err("SubRipTime index out of range")),
        }
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn shift(
        &mut self,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let mut hours = 0;
        let mut minutes = 0;
        let mut seconds = 0;
        let mut milliseconds = 0;
        let mut ratio = None;

        let len = args.len();
        if len > 0 {
            hours = args.get_item(0)?.extract()?;
        }
        if len > 1 {
            minutes = args.get_item(1)?.extract()?;
        }
        if len > 2 {
            seconds = args.get_item(2)?.extract()?;
        }
        if len > 3 {
            milliseconds = args.get_item(3)?.extract()?;
        }

        if let Some(kw) = kwargs {
            if let Some(h) = kw.get_item("hours")? {
                hours = h.extract()?;
            }
            if let Some(m) = kw.get_item("minutes")? {
                minutes = m.extract()?;
            }
            if let Some(s) = kw.get_item("seconds")? {
                seconds = s.extract()?;
            }
            if let Some(ms) = kw.get_item("milliseconds")? {
                milliseconds = ms.extract()?;
            }
            if let Some(r) = kw.get_item("ratio")? {
                ratio = Some(r.extract()?);
            }
        }

        self.inner.shift(hours, minutes, seconds, milliseconds, ratio);
        Ok(())
    }

    #[classmethod]
    fn from_ordinal(_cls: &Bound<'_, PyType>, ordinal: i64) -> Self {
        Self {
            inner: RustSubRipTime::from_ordinal(ordinal),
        }
    }

    #[classmethod]
    fn from_string(_cls: &Bound<'_, PyType>, source: &str) -> PyResult<Self> {
        RustSubRipTime::from_string(source)
            .map(|t| Self { inner: t })
            .map_err(to_py_err)
    }

    #[classmethod]
    fn from_time(_cls: &Bound<'_, PyType>, source: &Bound<'_, PyAny>) -> PyResult<Self> {
        let hour: i64 = source.getattr("hour")?.extract()?;
        let minute: i64 = source.getattr("minute")?.extract()?;
        let second: i64 = source.getattr("second")?.extract()?;
        let microsecond: i64 = source.getattr("microsecond")?.extract()?;
        Ok(Self::new(hour, minute, second, microsecond / 1000))
    }

    #[classmethod]
    fn coerce(_cls: &Bound<'_, PyType>, other: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        coerce_time(other)
    }

    fn to_time<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let datetime_mod = PyModule::import_bound(py, "datetime")?;
        let time_cls = datetime_mod.getattr("time")?;
        let microsecond = self.milliseconds() * 1000;
        time_cls.call1((self.hours(), self.minutes(), self.seconds(), microsecond))
    }
}

#[pyclass(name = "SubRipItem", module = "pysrt")]
pub struct PySubRipItem {
    pub index: ItemIndex,
    pub start: Py<PySubRipTime>,
    pub end: Py<PySubRipTime>,
    pub text: String,
    pub position: String,
}

#[pymethods]
impl PySubRipItem {
    #[new]
    #[pyo3(signature = (index=None, start=None, end=None, text="", position=""))]
    fn new(
        py: Python<'_>,
        index: Option<&Bound<'_, PyAny>>,
        start: Option<&Bound<'_, PyAny>>,
        end: Option<&Bound<'_, PyAny>>,
        text: &str,
        position: &str,
    ) -> PyResult<Self> {
        let idx = if let Some(ind) = index {
            if ind.is_none() {
                ItemIndex::None
            } else if let Ok(i) = ind.extract::<i32>() {
                ItemIndex::Int(i)
            } else if let Ok(s) = ind.extract::<String>() {
                ItemIndex::Str(s)
            } else {
                ItemIndex::Int(0)
            }
        } else {
            ItemIndex::Int(0)
        };

        let start_time = if let Some(s) = start {
            coerce_time(s)?
        } else {
            Py::new(py, PySubRipTime::new(0, 0, 0, 0))?
        };

        let end_time = if let Some(e) = end {
            coerce_time(e)?
        } else {
            Py::new(py, PySubRipTime::new(0, 0, 0, 0))?
        };

        Ok(Self {
            index: idx,
            start: start_time,
            end: end_time,
            text: text.to_string(),
            position: position.to_string(),
        })
    }

    #[getter]
    fn index<'py>(&self, py: Python<'py>) -> PyObject {
        match &self.index {
            ItemIndex::Int(n) => n.to_object(py),
            ItemIndex::Str(s) => s.to_object(py),
            ItemIndex::None => py.None(),
        }
    }

    #[setter]
    fn set_index(&mut self, val: &Bound<'_, PyAny>) -> PyResult<()> {
        if val.is_none() {
            self.index = ItemIndex::None;
        } else if let Ok(i) = val.extract::<i32>() {
            self.index = ItemIndex::Int(i);
        } else if let Ok(s) = val.extract::<String>() {
            self.index = ItemIndex::Str(s);
        } else {
            self.index = ItemIndex::Int(0);
        }
        Ok(())
    }

    #[getter]
    fn start<'py>(&self, py: Python<'py>) -> Py<PySubRipTime> {
        self.start.clone_ref(py)
    }

    #[setter]
    fn set_start(&mut self, val: Py<PySubRipTime>) {
        self.start = val;
    }

    #[getter]
    fn end<'py>(&self, py: Python<'py>) -> Py<PySubRipTime> {
        self.end.clone_ref(py)
    }

    #[setter]
    fn set_end(&mut self, val: Py<PySubRipTime>) {
        self.end = val;
    }

    #[getter]
    fn text(&self) -> String {
        self.text.clone()
    }

    #[setter]
    fn set_text(&mut self, val: String) {
        self.text = val;
    }

    #[getter]
    fn position(&self) -> String {
        self.position.clone()
    }

    #[setter]
    fn set_position(&mut self, val: String) {
        self.position = val;
    }

    #[getter]
    fn duration(&self, py: Python<'_>) -> PySubRipTime {
        let s = self.start.borrow(py).inner;
        let e = self.end.borrow(py).inner;
        PySubRipTime {
            inner: RustSubRipTime::from_ordinal(e.ordinal - s.ordinal),
        }
    }

    #[getter]
    fn text_without_tags(&self, py: Python<'_>) -> String {
        let rust_item = self.to_rust_item(py);
        rust_item.text_without_tags()
    }

    #[getter]
    fn characters_per_second(&self, py: Python<'_>) -> f64 {
        let rust_item = self.to_rust_item(py);
        rust_item.characters_per_second()
    }

    fn __str__(&self, py: Python<'_>) -> String {
        self.to_rust_item(py).to_string()
    }

    fn __repr__(&self, py: Python<'_>) -> String {
        format!(
            "SubRipItem(index={}, start={}, end={}, text={:?}, position={:?})",
            self.index,
            self.start.borrow(py).__str__(),
            self.end.borrow(py).__str__(),
            self.text,
            self.position
        )
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> bool {
        if let Ok(other_item) = other.extract::<PyRef<PySubRipItem>>() {
            (self.start.borrow(py).inner, self.end.borrow(py).inner)
                == (other_item.start.borrow(py).inner, other_item.end.borrow(py).inner)
        } else {
            false
        }
    }

    fn __lt__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_item = other.extract::<PyRef<PySubRipItem>>()?;
        let s1 = self.start.borrow(py).inner.ordinal;
        let e1 = self.end.borrow(py).inner.ordinal;
        let s2 = other_item.start.borrow(py).inner.ordinal;
        let e2 = other_item.end.borrow(py).inner.ordinal;
        Ok((s1, e1) < (s2, e2))
    }

    fn __le__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_item = other.extract::<PyRef<PySubRipItem>>()?;
        let s1 = self.start.borrow(py).inner.ordinal;
        let e1 = self.end.borrow(py).inner.ordinal;
        let s2 = other_item.start.borrow(py).inner.ordinal;
        let e2 = other_item.end.borrow(py).inner.ordinal;
        Ok((s1, e1) <= (s2, e2))
    }

    fn __gt__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_item = other.extract::<PyRef<PySubRipItem>>()?;
        let s1 = self.start.borrow(py).inner.ordinal;
        let e1 = self.end.borrow(py).inner.ordinal;
        let s2 = other_item.start.borrow(py).inner.ordinal;
        let e2 = other_item.end.borrow(py).inner.ordinal;
        Ok((s1, e1) > (s2, e2))
    }

    fn __ge__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let other_item = other.extract::<PyRef<PySubRipItem>>()?;
        let s1 = self.start.borrow(py).inner.ordinal;
        let e1 = self.end.borrow(py).inner.ordinal;
        let s2 = other_item.start.borrow(py).inner.ordinal;
        let e2 = other_item.end.borrow(py).inner.ordinal;
        Ok((s1, e1) >= (s2, e2))
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn shift(
        &mut self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        self.start.borrow_mut(py).shift(args, kwargs)?;
        self.end.borrow_mut(py).shift(args, kwargs)?;
        Ok(())
    }

    #[classmethod]
    fn from_string(_cls: &Bound<'_, PyType>, py: Python<'_>, source: &str) -> PyResult<Self> {
        RustSubRipItem::from_string(source)
            .map_err(to_py_err)
            .and_then(|i| Self::from_rust_item(py, i))
    }

    #[classmethod]
    fn from_lines(_cls: &Bound<'_, PyType>, py: Python<'_>, lines: Vec<String>) -> PyResult<Self> {
        let str_lines: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        RustSubRipItem::from_lines(str_lines)
            .map_err(to_py_err)
            .and_then(|i| Self::from_rust_item(py, i))
    }

    #[classmethod]
    fn split_timestamps(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        line: &str,
    ) -> PyResult<(Py<PySubRipTime>, Py<PySubRipTime>, String)> {
        RustSubRipItem::split_timestamps(line)
            .map_err(to_py_err)
            .and_then(|(s, e, p)| {
                Ok((
                    Py::new(py, PySubRipTime { inner: s })?,
                    Py::new(py, PySubRipTime { inner: e })?,
                    p,
                ))
            })
    }
}

impl PySubRipItem {
    pub fn to_rust_item(&self, py: Python<'_>) -> RustSubRipItem {
        RustSubRipItem {
            index: self.index.clone(),
            start: self.start.borrow(py).inner,
            end: self.end.borrow(py).inner,
            text: self.text.clone(),
            position: self.position.clone(),
        }
    }

    pub fn from_rust_item(py: Python<'_>, item: RustSubRipItem) -> PyResult<Self> {
        Ok(Self {
            index: item.index,
            start: Py::new(py, PySubRipTime { inner: item.start })?,
            end: Py::new(py, PySubRipTime { inner: item.end })?,
            text: item.text,
            position: item.position,
        })
    }
}

#[pyclass(name = "SubRipFile", module = "pysrt", sequence)]
pub struct PySubRipFile {
    pub items: Vec<Py<PySubRipItem>>,
    pub eol: String,
    pub path: Option<String>,
    pub encoding: String,
}

#[pymethods]
impl PySubRipFile {
    #[classattr]
    const ERROR_PASS: i32 = 0;
    #[classattr]
    const ERROR_LOG: i32 = 1;
    #[classattr]
    const ERROR_RAISE: i32 = 2;

    #[new]
    #[pyo3(signature = (items=None, eol=None, path=None, encoding="utf-8"))]
    fn new(
        py: Python<'_>,
        items: Option<Vec<Py<PySubRipItem>>>,
        eol: Option<String>,
        path: Option<String>,
        encoding: &str,
    ) -> PyResult<Self> {
        let os_mod = PyModule::import_bound(py, "os")?;
        let linesep: String = os_mod.getattr("linesep")?.extract()?;
        Ok(Self {
            items: items.unwrap_or_default(),
            eol: eol.unwrap_or(linesep),
            path,
            encoding: encoding.to_string(),
        })
    }

    #[getter]
    fn eol(&self) -> String {
        self.eol.clone()
    }

    #[setter]
    fn set_eol(&mut self, val: String) {
        self.eol = val;
    }

    #[getter]
    fn path(&self) -> Option<String> {
        self.path.clone()
    }

    #[setter]
    fn set_path(&mut self, val: Option<String>) {
        self.path = val;
    }

    #[getter]
    fn encoding(&self) -> String {
        self.encoding.clone()
    }

    #[setter]
    fn set_encoding(&mut self, val: String) {
        self.encoding = val;
    }

    #[getter]
    fn text(&self, py: Python<'_>) -> PyResult<String> {
        let mut texts = Vec::new();
        for item_py in &self.items {
            let item_ref = item_py.borrow(py);
            texts.push(item_ref.text.clone());
        }
        Ok(texts.join("\n"))
    }

    fn __len__(&self) -> usize {
        self.items.len()
    }

    fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        idx_or_slice: &Bound<'py, PyAny>,
    ) -> PyResult<PyObject> {
        if let Ok(idx) = idx_or_slice.extract::<isize>() {
            let len = self.items.len() as isize;
            let norm_idx = if idx < 0 { len + idx } else { idx };
            if norm_idx < 0 || norm_idx >= len {
                return Err(PyIndexError::new_err("SubRipFile index out of range"));
            }
            Ok(self.items[norm_idx as usize].clone_ref(py).to_object(py))
        } else if let Ok(slice) = idx_or_slice.downcast::<PySlice>() {
            let len = self.items.len();
            let indices = slice.indices(len as isize)?;
            let mut sliced_items = Vec::new();
            let mut i = indices.start;
            while (indices.step > 0 && i < indices.stop)
                || (indices.step < 0 && i > indices.stop)
            {
                if i >= 0 && (i as usize) < len {
                    sliced_items.push(self.items[i as usize].clone_ref(py));
                }
                i += indices.step;
            }
            let new_file = PySubRipFile {
                items: sliced_items,
                eol: self.eol.clone(),
                path: self.path.clone(),
                encoding: self.encoding.clone(),
            };
            Ok(Py::new(py, new_file)?.to_object(py))
        } else {
            Err(PyTypeError::new_err("Invalid index type"))
        }
    }

    fn __setitem__(&mut self, idx: isize, val: Py<PySubRipItem>) -> PyResult<()> {
        let len = self.items.len() as isize;
        let norm_idx = if idx < 0 { len + idx } else { idx };
        if norm_idx < 0 || norm_idx >= len {
            return Err(PyIndexError::new_err("SubRipFile index out of range"));
        }
        self.items[norm_idx as usize] = val;
        Ok(())
    }

    fn __delitem__(&mut self, idx: isize) -> PyResult<()> {
        let len = self.items.len() as isize;
        let norm_idx = if idx < 0 { len + idx } else { idx };
        if norm_idx < 0 || norm_idx >= len {
            return Err(PyIndexError::new_err("SubRipFile index out of range"));
        }
        self.items.remove(norm_idx as usize);
        Ok(())
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<PySubRipFileIter>> {
        let items: Vec<Py<PySubRipItem>> = slf
            .items
            .iter()
            .map(|item| item.clone_ref(slf.py()))
            .collect();
        Py::new(
            slf.py(),
            PySubRipFileIter {
                items,
                index: 0,
            },
        )
    }

    fn append(&mut self, item: Py<PySubRipItem>) {
        self.items.push(item);
    }

    fn extend(&mut self, items: Vec<Py<PySubRipItem>>) {
        self.items.extend(items);
    }

    #[pyo3(signature = (idx=None))]
    fn pop(&mut self, _py: Python<'_>, idx: Option<isize>) -> PyResult<Py<PySubRipItem>> {
        let len = self.items.len() as isize;
        if len == 0 {
            return Err(PyIndexError::new_err("pop from empty SubRipFile"));
        }
        let target = idx.unwrap_or(-1);
        let norm_idx = if target < 0 { len + target } else { target };
        if norm_idx < 0 || norm_idx >= len {
            return Err(PyIndexError::new_err("pop index out of range"));
        }
        Ok(self.items.remove(norm_idx as usize))
    }

    fn sort(&mut self, py: Python<'_>) {
        self.items.sort_by(|a, b| {
            let a_ref = a.borrow(py);
            let b_ref = b.borrow(py);
            let a_s = a_ref.start.borrow(py).inner;
            let a_e = a_ref.end.borrow(py).inner;
            let b_s = b_ref.start.borrow(py).inner;
            let b_e = b_ref.end.borrow(py).inner;
            (a_s, a_e).cmp(&(b_s, b_e))
        });
    }

    fn clean_indexes(&mut self, py: Python<'_>) {
        self.sort(py);
        for (i, item) in self.items.iter_mut().enumerate() {
            let mut item_mut = item.borrow_mut(py);
            item_mut.index = ItemIndex::Int((i + 1) as i32);
        }
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn shift(
        &mut self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        for item in &self.items {
            let mut item_mut = item.borrow_mut(py);
            item_mut.shift(py, args, kwargs)?;
        }
        Ok(())
    }

    #[pyo3(signature = (path=None, encoding=None, eol=None))]
    fn save(
        &self,
        py: Python<'_>,
        path: Option<&str>,
        encoding: Option<&str>,
        eol: Option<&str>,
    ) -> PyResult<()> {
        let target_path = path
            .map(|s| s.to_string())
            .or_else(|| self.path.clone())
            .ok_or_else(|| {
                InvalidItem::new_err("No file path specified for save")
            })?;
        let eol_str = eol.unwrap_or(&self.eol);
        let enc = encoding.unwrap_or(&self.encoding);

        let mut texts = Vec::new();
        for item_py in &self.items {
            let item_ref = item_py.borrow(py);
            let s_time = item_ref.start.borrow(py).__str__();
            let e_time = item_ref.end.borrow(py).__str__();
            let pos = if item_ref.position.trim().is_empty() {
                String::new()
            } else {
                format!(" {}", item_ref.position)
            };
            let mut block = format!("{}\n{} --> {}{}\n{}\n", item_ref.index, s_time, e_time, pos, item_ref.text);
            if !block.ends_with("\n\n") {
                block.push('\n');
            }
            if eol_str != "\n" {
                block = block.replace('\n', eol_str);
            }
            texts.push(block);
        }
        let content = texts.join("");

        let codecs_mod = PyModule::import_bound(py, "codecs")?;
        let open_fn = codecs_mod.getattr("open")?;
        let file_obj = open_fn.call1((target_path, "w", enc))?;
        file_obj.call_method1("write", (content,))?;
        file_obj.call_method0("close")?;
        Ok(())
    }

    #[pyo3(signature = (starts_before=None, starts_after=None, ends_before=None, ends_after=None))]
    fn slice(
        &self,
        py: Python<'_>,
        starts_before: Option<&Bound<'_, PyAny>>,
        starts_after: Option<&Bound<'_, PyAny>>,
        ends_before: Option<&Bound<'_, PyAny>>,
        ends_after: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<Self>> {
        let sb = starts_before
            .map(|t| coerce_time(t))
            .transpose()?
            .map(|t| t.borrow(py).inner);
        let sa = starts_after
            .map(|t| coerce_time(t))
            .transpose()?
            .map(|t| t.borrow(py).inner);
        let eb = ends_before
            .map(|t| coerce_time(t))
            .transpose()?
            .map(|t| t.borrow(py).inner);
        let ea = ends_after
            .map(|t| coerce_time(t))
            .transpose()?
            .map(|t| t.borrow(py).inner);

        let mut matched = Vec::new();
        for item_py in &self.items {
            let item_ref = item_py.borrow(py);
            let mut keep = true;
            if let Some(t) = sb {
                if item_ref.start.borrow(py).inner >= t {
                    keep = false;
                }
            }
            if let Some(t) = sa {
                if item_ref.start.borrow(py).inner <= t {
                    keep = false;
                }
            }
            if let Some(t) = eb {
                if item_ref.end.borrow(py).inner >= t {
                    keep = false;
                }
            }
            if let Some(t) = ea {
                if item_ref.end.borrow(py).inner <= t {
                    keep = false;
                }
            }
            if keep {
                matched.push(item_py.clone_ref(py));
            }
        }
        Py::new(
            py,
            Self {
                items: matched,
                eol: self.eol.clone(),
                path: self.path.clone(),
                encoding: self.encoding.clone(),
            },
        )
    }

    #[pyo3(signature = (timestamp=None, **kwargs))]
    fn at(
        &self,
        py: Python<'_>,
        timestamp: Option<&Bound<'_, PyAny>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let t = if let Some(ts) = timestamp {
            coerce_time(ts)?.borrow(py).inner
        } else if let Some(kw) = kwargs {
            coerce_time(kw.as_any())?.borrow(py).inner
        } else {
            RustSubRipTime::new(0, 0, 0, 0)
        };

        let mut matched = Vec::new();
        for item_py in &self.items {
            let item_ref = item_py.borrow(py);
            let s = item_ref.start.borrow(py).inner;
            let e = item_ref.end.borrow(py).inner;
            if t >= s && t <= e {
                matched.push(item_py.clone_ref(py));
            }
        }
        Py::new(
            py,
            Self {
                items: matched,
                eol: self.eol.clone(),
                path: self.path.clone(),
                encoding: self.encoding.clone(),
            },
        )
    }

    #[classmethod]
    #[pyo3(signature = (path="", encoding=None, _error_handling=0))]
    fn open(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        path: &str,
        encoding: Option<&str>,
        _error_handling: i32,
    ) -> PyResult<Py<Self>> {
        // /dev/null → empty file (also handles Windows where /dev/null does not exist).
        if path == "/dev/null" {
            let os_mod = PyModule::import_bound(py, "os")?;
            let linesep: String = os_mod.getattr("linesep")?.extract()?;
            return Py::new(py, Self {
                items: vec![],
                eol: linesep,
                path: None,
                encoding: "utf_8".to_string(),
            });
        }

        let codecs_mod = PyModule::import_bound(py, "codecs")?;

        // Determine the effective codec and canonical encoding name.
        // When no encoding is given: sniff BOM bytes to pick the right codec,
        // then fall back to strict utf-8 (which raises UnicodeDecodeError for non-UTF-8 files).
        let (effective_enc, enc_name) = if let Some(enc) = encoding {
            // User provided encoding — just normalise the name.
            let lookup_fn = codecs_mod.getattr("lookup")?;
            let raw: String = lookup_fn.call1((enc,))?.getattr("name")?.extract()?;
            let norm = match raw.as_str() {
                "utf-8" => "utf_8".to_string(),
                "windows-1252" => "cp1252".to_string(),
                other => other.to_string(),
            };
            (enc.to_string(), norm)
        } else {
            // No encoding — detect BOM from first raw bytes.
            let builtins = py.import_bound("builtins")?;
            let raw_file = builtins.getattr("open")?.call1((path, "rb"))?;
            let header_bytes: Vec<u8> = raw_file.call_method1("read", (4i32,))?.extract()?;
            raw_file.call_method0("close")?;

            if header_bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) || header_bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
                // UTF-32 (LE or BE) — use utf-32 codec which handles both BOMs
                ("utf-32".to_string(), "utf_32".to_string())
            } else if header_bytes.starts_with(&[0xFF, 0xFE]) || header_bytes.starts_with(&[0xFE, 0xFF]) {
                // UTF-16 (LE or BE) — use utf-16 codec which handles both BOMs
                ("utf-16".to_string(), "utf_16".to_string())
            } else if header_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
                // UTF-8 BOM — use utf-8-sig which strips the BOM
                ("utf-8-sig".to_string(), "utf_8".to_string())
            } else {
                // No BOM → strict UTF-8 (raises UnicodeDecodeError for non-UTF-8 files)
                ("utf_8".to_string(), "utf_8".to_string())
            }
        };

        let open_fn = codecs_mod.getattr("open")?;
        let file_obj = open_fn.call1((path, "r", effective_enc.as_str()))?;
        let content: String = file_obj.call_method0("read")?.extract()?;
        file_obj.call_method0("close")?;

        // Detect EOL from decoded content
        let eol = RustSubRipFile::guess_eol(&content);

        let err_mode = match _error_handling {
            0 => ErrorHandling::Pass,
            1 => ErrorHandling::Log,
            _ => ErrorHandling::Raise,
        };

        let rust_file =
            RustSubRipFile::from_string_with_error_handling(&content, err_mode)
                .map_err(to_py_err)?;
        let mut items = Vec::with_capacity(rust_file.items.len());
        for rust_item in rust_file.items {
            let py_item = Py::new(py, PySubRipItem::from_rust_item(py, rust_item)?)?;
            items.push(py_item);
        }
        Py::new(
            py,
            Self {
                items,
                eol,
                path: Some(path.to_string()),
                encoding: enc_name,
            },
        )
    }

    #[classmethod]
    #[pyo3(signature = (source, **kwargs))]
    fn from_string(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        source: &str,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<Self>> {
        let mut eol = None;
        let mut encoding = "utf-8".to_string();
        let mut path = None;
        let mut err_mode = ErrorHandling::Raise;

        if let Some(kw) = kwargs {
            if let Some(e) = kw.get_item("eol")? {
                eol = Some(e.extract::<String>()?);
            }
            if let Some(enc) = kw.get_item("encoding")? {
                encoding = enc.extract::<String>()?;
            }
            if let Some(p) = kw.get_item("path")? {
                path = Some(p.extract::<String>()?);
            }
            if let Some(m) = kw.get_item("error_handling")? {
                match m.extract::<i32>()? {
                    0 => err_mode = ErrorHandling::Pass,
                    1 => err_mode = ErrorHandling::Log,
                    _ => err_mode = ErrorHandling::Raise,
                }
            }
        }

        let rust_file =
            RustSubRipFile::from_string_with_error_handling(source, err_mode).map_err(to_py_err)?;

        let mut items = Vec::with_capacity(rust_file.items.len());
        for rust_item in rust_file.items {
            let py_item = Py::new(py, PySubRipItem::from_rust_item(py, rust_item)?)?;
            items.push(py_item);
        }
        Py::new(
            py,
            Self {
                items,
                eol: eol.unwrap_or(rust_file.eol),
                path: path.or(rust_file.path),
                encoding,
            },
        )
    }

    #[classmethod]
    #[pyo3(signature = (source_file, error_handling=0))]
    fn stream(
        _cls: &Bound<'_, PyType>,
        py: Python<'_>,
        source_file: &Bound<'_, PyAny>,
        error_handling: i32,
    ) -> PyResult<Py<PySubRipFileIter>> {
        let mut full_str = String::new();
        if let Ok(iter) = source_file.iter() {
            for item in iter {
                let line: String = item?.extract()?;
                full_str.push_str(&line);
            }
        } else if let Ok(s) = source_file.extract::<String>() {
            full_str = s;
        } else if let Ok(lines) = source_file.call_method0("readlines") {
            for item in lines.iter()? {
                let line: String = item?.extract()?;
                full_str.push_str(&line);
            }
        }

        let err_mode = match error_handling {
            0 => ErrorHandling::Pass,
            1 => ErrorHandling::Log,
            _ => ErrorHandling::Raise,
        };

        let rust_file = RustSubRipFile::from_string_with_error_handling(&full_str, err_mode)
            .map_err(to_py_err)?;
        let mut items = Vec::with_capacity(rust_file.items.len());
        for rust_item in rust_file.items {
            let py_item = Py::new(py, PySubRipItem::from_rust_item(py, rust_item)?)?;
            items.push(py_item);
        }

        Py::new(
            py,
            PySubRipFileIter { items, index: 0 },
        )
    }
}

#[pyclass]
pub struct PySubRipFileIter {
    items: Vec<Py<PySubRipItem>>,
    index: usize,
}

#[pymethods]
impl PySubRipFileIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self, py: Python<'_>) -> Option<Py<PySubRipItem>> {
        if self.index < self.items.len() {
            let item = self.items[self.index].clone_ref(py);
            self.index += 1;
            Some(item)
        } else {
            None
        }
    }
}

#[pyfunction]
#[pyo3(signature = (path, encoding=None, error_handling=None))]
fn open(
    py: Python<'_>,
    path: &str,
    encoding: Option<&str>,
    error_handling: Option<i32>,
) -> PyResult<Py<PySubRipFile>> {
    PySubRipFile::open(
        &py.get_type_bound::<PySubRipFile>(),
        py,
        path,
        encoding,
        error_handling.unwrap_or(0),
    )
}

#[pyfunction]
#[pyo3(signature = (source, **kwargs))]
fn from_string(
    py: Python<'_>,
    source: &str,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PySubRipFile>> {
    PySubRipFile::from_string(&py.get_type_bound::<PySubRipFile>(), py, source, kwargs)
}

#[pyfunction]
#[pyo3(signature = (source_file, error_handling=0))]
fn stream(
    py: Python<'_>,
    source_file: &Bound<'_, PyAny>,
    error_handling: i32,
) -> PyResult<Py<PySubRipFileIter>> {
    PySubRipFile::stream(
        &py.get_type_bound::<PySubRipFile>(),
        py,
        source_file,
        error_handling,
    )
}

#[pymodule]
pub fn pysrt(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PySubRipTime>()?;
    m.add_class::<PySubRipItem>()?;
    m.add_class::<PySubRipFile>()?;
    m.add_class::<PyTimeDescriptor>()?;
    m.add_function(wrap_pyfunction!(open, m)?)?;
    m.add_function(wrap_pyfunction!(from_string, m)?)?;
    m.add_function(wrap_pyfunction!(stream, m)?)?;

    m.add("Error", py.get_type_bound::<Error>())?;
    m.add("InvalidItem", py.get_type_bound::<InvalidItem>())?;
    m.add(
        "InvalidTimeString",
        py.get_type_bound::<InvalidTimeString>(),
    )?;

    m.add("ERROR_PASS", 0)?;
    m.add("ERROR_LOG", 1)?;
    m.add("ERROR_RAISE", 2)?;
    m.add("SUPPORT_UTF_32_LE", true)?;
    m.add("SUPPORT_UTF_32_BE", true)?;
    m.add("VERSION", (1, 1, 2))?;
    m.add("VERSION_STRING", "1.1.2")?;

    let time_cls = py.get_type_bound::<PySubRipTime>();
    time_cls.setattr("hours", Py::new(py, PyTimeDescriptor { field: 0 })?)?;
    time_cls.setattr("minutes", Py::new(py, PyTimeDescriptor { field: 1 })?)?;
    time_cls.setattr("seconds", Py::new(py, PyTimeDescriptor { field: 2 })?)?;
    time_cls.setattr("milliseconds", Py::new(py, PyTimeDescriptor { field: 3 })?)?;
    time_cls.setattr("ordinal", Py::new(py, PyTimeDescriptor { field: 4 })?)?;

    let compat_mod = PyModule::new_bound(py, "compat")?;
    compat_mod.add("str", py.get_type_bound::<PyString>())?;
    compat_mod.add("basestring", py.get_type_bound::<PyString>())?;
    // pysrt.compat.open is Python's built-in io.open (not our subtitle open),
    // so tests that do `from pysrt.compat import open` get real file-open semantics.
    let io_mod = py.import_bound("io")?;
    let io_open = io_mod.getattr("open")?;
    compat_mod.add("open", io_open)?;
    m.add_submodule(&compat_mod)?;
    py.import_bound("sys")?
        .getattr("modules")?
        .set_item("pysrt.compat", compat_mod)?;

    Ok(())
}
