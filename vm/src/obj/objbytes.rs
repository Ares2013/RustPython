use std::cell::Cell;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use num_traits::ToPrimitive;

use crate::function::{OptionalArg, PyFuncArgs};
use crate::pyobject::{
    PyContext, PyIteratorValue, PyObjectRef, PyRef, PyResult, PyValue, TypeProtocol,
};
use crate::vm::VirtualMachine;

use super::objint;
use super::objtype::{self, PyClassRef};

#[derive(Debug)]
pub struct PyBytes {
    value: Vec<u8>,
}
type PyBytesRef = PyRef<PyBytes>;

impl PyBytes {
    pub fn new(data: Vec<u8>) -> Self {
        PyBytes { value: data }
    }
}

impl Deref for PyBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.value
    }
}

impl PyValue for PyBytes {
    fn class(vm: &VirtualMachine) -> PyClassRef {
        vm.ctx.bytes_type()
    }
}

// Binary data support

// Fill bytes class methods:
pub fn init(context: &PyContext) {
    let bytes_type = context.bytes_type.as_object();

    let bytes_doc =
        "bytes(iterable_of_ints) -> bytes\n\
         bytes(string, encoding[, errors]) -> bytes\n\
         bytes(bytes_or_buffer) -> immutable copy of bytes_or_buffer\n\
         bytes(int) -> bytes object of size given by the parameter initialized with null bytes\n\
         bytes() -> empty bytes object\n\nConstruct an immutable array of bytes from:\n  \
         - an iterable yielding integers in range(256)\n  \
         - a text string encoded using the specified encoding\n  \
         - any object implementing the buffer API.\n  \
         - an integer";

    extend_class!(context, bytes_type, {
        "__new__" => context.new_rustfunc(bytes_new),
        "__eq__" => context.new_rustfunc(PyBytesRef::eq),
        "__lt__" => context.new_rustfunc(PyBytesRef::lt),
        "__le__" => context.new_rustfunc(PyBytesRef::le),
        "__gt__" => context.new_rustfunc(PyBytesRef::gt),
        "__ge__" => context.new_rustfunc(PyBytesRef::ge),
        "__hash__" => context.new_rustfunc(PyBytesRef::hash),
        "__repr__" => context.new_rustfunc(PyBytesRef::repr),
        "__len__" => context.new_rustfunc(PyBytesRef::len),
        "__iter__" => context.new_rustfunc(PyBytesRef::iter),
        "__doc__" => context.new_str(bytes_doc.to_string())
    });
}

fn bytes_new(
    cls: PyClassRef,
    val_option: OptionalArg<PyObjectRef>,
    vm: &VirtualMachine,
) -> PyResult<PyBytesRef> {
    // Create bytes data:
    let value = if let OptionalArg::Present(ival) = val_option {
        let elements = vm.extract_elements(&ival)?;
        let mut data_bytes = vec![];
        for elem in elements.iter() {
            let v = objint::to_int(vm, elem, 10)?;
            data_bytes.push(v.to_u8().unwrap());
        }
        data_bytes
    // return Err(vm.new_type_error("Cannot construct bytes".to_string()));
    } else {
        vec![]
    };

    PyBytes::new(value).into_ref_with_type(vm, cls)
}

impl PyBytesRef {
    fn eq(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(
            vm,
            args,
            required = [(a, Some(vm.ctx.bytes_type())), (b, None)]
        );

        let result = if objtype::isinstance(b, &vm.ctx.bytes_type()) {
            get_value(a).to_vec() == get_value(b).to_vec()
        } else {
            false
        };
        Ok(vm.ctx.new_bool(result))
    }

    fn ge(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(
            vm,
            args,
            required = [(a, Some(vm.ctx.bytes_type())), (b, None)]
        );

        let result = if objtype::isinstance(b, &vm.ctx.bytes_type()) {
            get_value(a).to_vec() >= get_value(b).to_vec()
        } else {
            return Err(vm.new_type_error(format!("Cannot compare {} and {} using '>'", a, b)));
        };
        Ok(vm.ctx.new_bool(result))
    }

    fn gt(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(
            vm,
            args,
            required = [(a, Some(vm.ctx.bytes_type())), (b, None)]
        );

        let result = if objtype::isinstance(b, &vm.ctx.bytes_type()) {
            get_value(a).to_vec() > get_value(b).to_vec()
        } else {
            return Err(vm.new_type_error(format!("Cannot compare {} and {} using '>='", a, b)));
        };
        Ok(vm.ctx.new_bool(result))
    }

    fn le(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(
            vm,
            args,
            required = [(a, Some(vm.ctx.bytes_type())), (b, None)]
        );

        let result = if objtype::isinstance(b, &vm.ctx.bytes_type()) {
            get_value(a).to_vec() <= get_value(b).to_vec()
        } else {
            return Err(vm.new_type_error(format!("Cannot compare {} and {} using '<'", a, b)));
        };
        Ok(vm.ctx.new_bool(result))
    }

    fn lt(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(
            vm,
            args,
            required = [(a, Some(vm.ctx.bytes_type())), (b, None)]
        );

        let result = if objtype::isinstance(b, &vm.ctx.bytes_type()) {
            get_value(a).to_vec() < get_value(b).to_vec()
        } else {
            return Err(vm.new_type_error(format!("Cannot compare {} and {} using '<='", a, b)));
        };
        Ok(vm.ctx.new_bool(result))
    }

    fn len(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(vm, args, required = [(a, Some(vm.ctx.bytes_type()))]);

        let byte_vec = get_value(a).to_vec();
        Ok(vm.ctx.new_int(byte_vec.len()))
    }

    fn hash(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(vm, args, required = [(zelf, Some(vm.ctx.bytes_type()))]);
        let data = get_value(zelf);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();
        Ok(vm.ctx.new_int(hash))
    }

    fn repr(vm: &VirtualMachine, args: PyFuncArgs) -> PyResult {
        arg_check!(vm, args, required = [(obj, Some(vm.ctx.bytes_type()))]);
        let value = get_value(obj);
        let data = String::from_utf8(value.to_vec()).unwrap();
        Ok(vm.new_str(format!("b'{}'", data)))
    }

    fn iter(obj: PyBytesRef, _vm: &VirtualMachine) -> PyIteratorValue {
        PyIteratorValue {
            position: Cell::new(0),
            iterated_obj: obj.into_object(),
        }
    }
}

pub fn get_value<'a>(obj: &'a PyObjectRef) -> impl Deref<Target = Vec<u8>> + 'a {
    &obj.payload::<PyBytes>().unwrap().value
}
