//! Common traits and types for Magic DOM structs.

/// The proper name is the name of the struct generated by the MagicDom macro.
/// The class_name is the name of the JSClass generated by the MagicDom macro.
/// The ps_arr_name is the name of the JSPropertySpec array generated by the MagicDom macro.
/// The constructor name is the name of the constructor for the JSClass generated by the
/// MagicDom macro.
/// The mod_name is the name of the private module.
/// For example, suppose the name of the dom struct is DOMPoint($name) and the upper
/// case version is DOMPOINT($NAME). We will put the following in the magic_dom!:
/// ```
///     magic_dom! {
///         DOMPoint,
///         DOMPOINT_CLASS, /// the format for the class name would be $NAME + _CLASS
///         DOMPOINT_PS_ARR, /// the format for the JSPropertySpec array would be $NAME + _PS_ARR
///         DOMPoint_constructor, /// the format for the constructor would be $name + _constructor
///         /// the format for the private module name would be magic_dom_spec_ + $name
///         magic_dom_spec_DOMPoint,
///         /// the spec name format would be $name + _spec. the _ is needed in the MagicDom to
///         /// extract the $name.
///         struct DOMPoint_spec {
///             x: f64,
///             y: f64,
///             z: f64,
///             w: f64,
///         }
///     }
/// ```
#[macro_export]
macro_rules! magic_dom {
    (   $name:ident,
        $class_name:ident,
        $constructor_name:ident,
        $mod_name:ident,
        struct $spec_name:ident {
            $(
                $field:ident : $field_ty:ty,
            )*
        }
    ) => {
        mod $mod_name {
            #[allow(dead_code)]
            #[allow(non_camel_case_types)]
            #[derive(MagicDom)]
            struct $spec_name {
                $(
                    $field : $field_ty,
                )*
            }
        }
        pub use self::$mod_name::$name as $name;
        pub use self::$mod_name::$constructor_name as $constructor_name;
        pub use self::$mod_name::$class_name as $class_name;
    }
}

#[macro_export]
macro_rules! get_js_arg {
    ($val:ident, $cx:ident, $call_args:ident, $arg_idx:expr, $conversion_behavior:expr) => {
        let $val = match FromJSValConvertible::from_jsval($cx, $call_args.index($arg_idx), $conversion_behavior) {
            Ok(num) => {
                match num {
                    ConversionResult::Success(v) => v,
                    ConversionResult::Failure(why) => {
                        JS_ReportErrorASCII($cx, b"Should never put anything into a slot that we \
                                                   can't convert from JS Value\0".as_ptr()
                                            as *const libc::c_char);
                        debug!("{}", why);
                        return false;
                    }
                }
            },
            Err(_) => {
                JS_ReportErrorASCII($cx, b"Can't recognize arg\0".as_ptr() as *const libc::c_char);
                debug!("Arg index: {}", $arg_idx);
                return false;
            },
        };
    }
}

#[macro_export]
macro_rules! js_getter {
    ($js_getter_name:ident, $getter_name:ident, $name:ident) => {
        pub extern "C" fn $js_getter_name (cx: *mut JSContext, argc: u32, vp: *mut JS::Value)
                                           -> bool {
            let res = unsafe {
                let call_args = CreateCallArgsFromVp(argc, vp);
                if call_args._base.argc_ != 0 {
                    JS_ReportErrorASCII(cx, b"getter doesn't require any arguments\0".as_ptr()
                                        as *const libc::c_char);
                    return false;
                }
                let obj = match $name::check_this(cx, &call_args) {
                    Some(obj_) => obj_,
                    None => {
                        JS_ReportErrorASCII(cx, b"Can't convert JSObject\0".as_ptr()
                                            as *const libc::c_char);
                        return false;
                    },
                };
                let val = obj.$getter_name(cx);
                val.to_jsval(cx, call_args.rval());
                true
            };
            res
        }
    }
}

#[macro_export]
macro_rules! js_setter {
    ($js_setter_name:ident, $setter_name:ident, $name:ident, $conversion_behavior:expr) => {
        pub extern "C" fn $js_setter_name (cx: *mut JSContext, argc: u32, vp: *mut JS::Value)
                                           -> bool {
            let res = unsafe {
                let call_args = CreateCallArgsFromVp(argc, vp);
                if call_args._base.argc_ != 1 {
                    JS_ReportErrorASCII(cx, b"setter requires exactly 1 arguments\0".as_ptr()
                                        as *const libc::c_char);
                    return false;
                }
                let obj = match $name::check_this(cx, &call_args) {
                    Some(obj_) => obj_,
                    None => {
                        JS_ReportErrorASCII(cx, b"Can't convert JSObject\0".as_ptr()
                                            as *const libc::c_char);
                        return false;
                    },
                };
                get_js_arg!(v, cx, call_args, 0, $conversion_behavior);
                obj.$setter_name(cx, v);
                true
            };
            res
        }
    }
}

#[macro_export]
macro_rules! gen_getter_inherit {
    ($getter_name:ident, $ty:ty, $upcast:ident) => {
        pub unsafe fn $getter_name(self: &Self, cx: *mut JSContext) -> <$ty as ToFromJsSlots>::Target {
            let parent = self.$upcast();
            parent.$getter_name(cx)
        }
    }
}

#[macro_export]
macro_rules! gen_setter_inherit {
    ($setter_name:ident, $ty:ty, $upcast:ident) => {
        pub fn $setter_name(self: &Self, cx: *mut JSContext, t: $ty) {
            let parent = self.$upcast();
            parent.$setter_name(cx, t);
        }
    }
}
