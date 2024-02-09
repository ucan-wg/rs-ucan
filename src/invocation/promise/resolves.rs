use super::{PromiseAny, PromiseErr, PromiseOk};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Resolves<T> {
    Ok(PromiseOk<T>),
    Err(PromiseErr<T>),
}

impl<T> Resolves<Option<T>> {
    // FIXME Helpful for serde, maybe extract to a trait?
    pub fn resolved_none(&self) -> bool {
        match self {
            Resolves::Ok(p_ok) => match p_ok {
                PromiseOk::Fulfilled(None) => true,
                _ => false,
            },
            Resolves::Err(p_err) => match p_err {
                PromiseErr::Rejected(None) => true,
                _ => false,
            },
        }
    }
}

impl<T> Resolves<T> {
    pub fn new(val: T) -> Self {
        Resolves::Ok(PromiseOk::Fulfilled(val))
    }

    pub fn try_resolve(self) -> Result<T, Resolves<T>> {
        match self {
            Resolves::Ok(p_ok) => p_ok.try_resolve().map_err(Resolves::Ok),
            Resolves::Err(p_err) => p_err.try_resolve().map_err(Resolves::Err),
        }
    }

    // FIXME replace with variadic macro?
    // FIXME docs
    pub fn try_resolve_2<U>(
        t: Resolves<T>,
        u: Resolves<U>,
    ) -> Result<(T, U), (Resolves<T>, Resolves<U>)>
    where
        T: fmt::Debug,
        U: fmt::Debug,
    {
        if t.is_ready() && u.is_ready() {
            Ok((t.try_resolve().unwrap(), u.try_resolve().unwrap()))
        } else {
            Err((t, u))
        }
    }

    pub fn try_resolve_3<U, V>(
        t: Resolves<T>,
        u: Resolves<U>,
        v: Resolves<V>,
    ) -> Result<(T, U, V), (Resolves<T>, Resolves<U>, Resolves<V>)>
    where
        T: fmt::Debug,
        U: fmt::Debug,
        V: fmt::Debug,
    {
        if t.is_ready() && u.is_ready() && v.is_ready() {
            Ok((
                t.try_resolve().unwrap(),
                u.try_resolve().unwrap(),
                v.try_resolve().unwrap(),
            ))
        } else {
            Err((t, u, v))
        }
    }

    pub fn try_resolve_4<U, V, W>(
        t: Resolves<T>,
        u: Resolves<U>,
        v: Resolves<V>,
        w: Resolves<W>,
    ) -> Result<(T, U, V, W), (Resolves<T>, Resolves<U>, Resolves<V>, Resolves<W>)>
    where
        T: fmt::Debug,
        U: fmt::Debug,
        V: fmt::Debug,
        W: fmt::Debug,
    {
        if t.is_ready() && u.is_ready() && v.is_ready() && w.is_ready() {
            Ok((
                t.try_resolve().unwrap(),
                u.try_resolve().unwrap(),
                v.try_resolve().unwrap(),
                w.try_resolve().unwrap(),
            ))
        } else {
            Err((t, u, v, w))
        }
    }

    pub fn try_resolve_5<U, V, W, X>(
        t: Resolves<T>,
        u: Resolves<U>,
        v: Resolves<V>,
        w: Resolves<W>,
        x: Resolves<X>,
    ) -> Result<
        (T, U, V, W, X),
        (
            Resolves<T>,
            Resolves<U>,
            Resolves<V>,
            Resolves<W>,
            Resolves<X>,
        ),
    >
    where
        T: fmt::Debug,
        U: fmt::Debug,
        V: fmt::Debug,
        W: fmt::Debug,
        X: fmt::Debug,
    {
        if t.is_ready() && u.is_ready() && v.is_ready() && w.is_ready() && x.is_ready() {
            Ok((
                t.try_resolve().unwrap(),
                u.try_resolve().unwrap(),
                v.try_resolve().unwrap(),
                w.try_resolve().unwrap(),
                x.try_resolve().unwrap(),
            ))
        } else {
            Err((t, u, v, w, x))
        }
    }

    pub fn try_resolve_6<U, V, W, X, Y>(
        t: Resolves<T>,
        u: Resolves<U>,
        v: Resolves<V>,
        w: Resolves<W>,
        x: Resolves<X>,
        y: Resolves<Y>,
    ) -> Result<
        (T, U, V, W, X, Y),
        (
            Resolves<T>,
            Resolves<U>,
            Resolves<V>,
            Resolves<W>,
            Resolves<X>,
            Resolves<Y>,
        ),
    >
    where
        T: fmt::Debug,
        U: fmt::Debug,
        V: fmt::Debug,
        W: fmt::Debug,
        X: fmt::Debug,
        Y: fmt::Debug,
    {
        if [
            t.is_ready(),
            u.is_ready(),
            v.is_ready(),
            w.is_ready(),
            x.is_ready(),
            y.is_ready(),
        ]
        .iter()
        .all(|x| *x)
        {
            Ok((
                t.try_resolve().unwrap(),
                u.try_resolve().unwrap(),
                v.try_resolve().unwrap(),
                w.try_resolve().unwrap(),
                x.try_resolve().unwrap(),
                y.try_resolve().unwrap(),
            ))
        } else {
            Err((t, u, v, w, x, y))
        }
    }

    pub fn is_ready(&self) -> bool {
        match self {
            Resolves::Ok(p_ok) => match p_ok {
                PromiseOk::Fulfilled(_) => true,
                _ => false,
            },
            Resolves::Err(p_err) => match p_err {
                PromiseErr::Rejected(_) => true,
                _ => false,
            },
        }
    }

    pub fn map<U, F>(self, f: F) -> Resolves<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Resolves::Ok(p_ok) => Resolves::Ok(p_ok.map(f)),
            Resolves::Err(p_err) => Resolves::Err(p_err.map(f)),
        }
    }
}

impl<T: TryFrom<Ipld>> TryFrom<Ipld> for Resolves<T> {
    type Error = Ipld;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        // FIXME so much cloning
        let t = ipld.clone().try_into().map_err(|_| ipld.clone())?;
        Ok(PromiseOk::Fulfilled(t).into())
    }
}

impl<T> From<Resolves<T>> for Option<T> {
    fn from(r: Resolves<T>) -> Option<T> {
        match r {
            Resolves::Ok(p_ok) => p_ok.into(),
            Resolves::Err(p_err) => p_err.into(),
        }
    }
}

impl<T> From<Result<T, T>> for Resolves<T> {
    fn from(result: Result<T, T>) -> Self {
        match result {
            Ok(value) => Resolves::Ok(PromiseOk::Fulfilled(value)),
            Err(value) => Resolves::Err(PromiseErr::Rejected(value)),
        }
    }
}

impl<T: Into<Ipld>> From<Resolves<T>> for Ipld {
    fn from(resolves: Resolves<T>) -> Ipld {
        match resolves {
            Resolves::Ok(p_ok) => p_ok.into(),
            Resolves::Err(p_err) => p_err.into(),
        }
    }
}

impl<T> From<PromiseOk<T>> for Resolves<T> {
    fn from(ok: PromiseOk<T>) -> Self {
        Resolves::Ok(ok)
    }
}

impl<T> From<PromiseErr<T>> for Resolves<T> {
    fn from(err: PromiseErr<T>) -> Self {
        Resolves::Err(err)
    }
}

impl<T> TryFrom<Resolves<T>> for PromiseOk<T> {
    type Error = PromiseErr<T>;

    fn try_from(resolved: Resolves<T>) -> Result<Self, Self::Error> {
        match resolved {
            Resolves::Ok(ok) => Ok(ok),
            Resolves::Err(err) => Err(err),
        }
    }
}

impl<T> TryFrom<Resolves<T>> for PromiseErr<T> {
    type Error = PromiseOk<T>;

    fn try_from(resolved: Resolves<T>) -> Result<Self, Self::Error> {
        match resolved {
            Resolves::Ok(ok) => Err(ok),
            Resolves::Err(err) => Ok(err),
        }
    }
}

impl<T> From<Resolves<T>> for PromiseAny<T, T> {
    fn from(resolve: Resolves<T>) -> PromiseAny<T, T> {
        match resolve {
            Resolves::Ok(p_ok) => match p_ok {
                PromiseOk::Fulfilled(value) => PromiseAny::Fulfilled(value),
                PromiseOk::Pending(cid) => PromiseAny::Pending(cid),
            },
            Resolves::Err(p_err) => match p_err {
                PromiseErr::Rejected(err) => PromiseAny::Rejected(err),
                PromiseErr::Pending(cid) => PromiseAny::Pending(cid),
            },
        }
    }
}
