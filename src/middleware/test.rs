use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use {BeforeMiddleware, AfterMiddleware, AroundMiddleware, Handler, Chain};

use {Request, Response, WebResult, WebError};

use self::Kind::{Fine, Prob};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum Kind {
    Fine,
    Prob,
}

struct Middleware {
    normal: Arc<AtomicBool>,
    err: Arc<AtomicBool>,
    mode: Kind,
}

impl BeforeMiddleware for Middleware {
    fn before(&self, _: &mut Request) -> WebResult<()> {
        assert!(!self.normal.load(Ordering::Relaxed));
        self.normal.store(true, Ordering::Relaxed);
        println!("Before Mode: {:?}", self.mode);
        match self.mode {
            Fine => Ok(()),
            Prob => Err(error()),
        }
    }

    fn catch(&self, _: &mut Request, err: WebError) -> WebResult<()> {
        assert!(!self.err.load(Ordering::Relaxed));
        self.err.store(true, Ordering::Relaxed);
        println!("Catch Before Mode: {:?}", self.mode);
        match self.mode {
            Fine => Ok(()),
            Prob => Err(err),
        }
    }
}

impl AfterMiddleware for Middleware {
    fn after(&self, _: &mut Request, resp: Response) -> WebResult<Response> {
        assert!(!self.normal.load(Ordering::Relaxed));
        self.normal.store(true, Ordering::Relaxed);
        println!("After Mode: {:?}", self.mode);
        match self.mode {
            Fine => Ok(resp),
            Prob => Err(error()),
        }
    }

    fn catch(&self, _: &mut Request, err: WebError) -> WebResult<Response> {
        assert!(!self.err.load(Ordering::Relaxed));
        self.err.store(true, Ordering::Relaxed);
        println!("Catch After Mode: {:?}", self.mode);
        match self.mode {
            Fine => Ok(response()),
            Prob => Err(err),
        }
    }
}

impl Handler for Middleware {
    fn handle(&self, _: &mut Request) -> WebResult<Response> {
        assert!(!self.normal.load(Ordering::Relaxed));
        self.normal.store(true, Ordering::Relaxed);
        println!("Handler Mode: {:?}", self.mode);
        match self.mode {
            Fine => Ok(response()),
            Prob => Err(error()),
        }
    }
}

fn error() -> WebError {
    WebError
}

fn response() -> Response {
    Response
}

fn request() -> Request {
    Request
}


type ChainLike<T> = (Vec<T>, T, Vec<T>);
type Twice<T> = (T, T);

fn sharedbool(val: bool) -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(val))
}

fn derive_mode(val: bool) -> Kind {
    if val {
        Fine
    } else {
        Prob
    }    
}

fn counters(chain: &ChainLike<Kind>) -> ChainLike<Twice<Arc<AtomicBool>>> {
    let (ref befores, _, ref afters) = *chain;

    (befores.iter().map(|_| (sharedbool(false), sharedbool(false))).collect(),
     (sharedbool(false), sharedbool(false)),
    afters.iter().map(|_| (sharedbool(false), sharedbool(false))).collect())
}

fn into_middle(chain: ChainLike<Kind>, flags: &ChainLike<Twice<Arc<AtomicBool>>>) -> ChainLike<Middleware> {
    let (befores, handler, afters) = chain;
    let (ref flags_before, ref flag_handler, ref flags_after) = *flags;

    let before_middles: Vec<Middleware> = befores.iter().enumerate()
        .map(|(idx, &mode)| {
        Middleware {
            normal: flags_before[idx].0.clone(),
            err: flags_before[idx].1.clone(),
            mode: mode
        }
    }).collect();

    let handler_middle = Middleware {
        normal: flag_handler.0.clone(),
        err: flag_handler.1.clone(),
        mode: handler,
    };

    let after_middles: Vec<Middleware> = afters.iter().enumerate()
        .map(|(idx, &mode)| {
        Middleware {
            normal: flags_after[idx].0.clone(),
            err: flags_after[idx].1.clone(),
            mode: mode,
        }
    }).collect();

    (before_middles, handler_middle, after_middles)
}

fn to_mode(actuals: &ChainLike<Twice<Arc<AtomicBool>>>) -> ChainLike<Kind> {
    let (ref before_middle, ref handler_middle, ref after_middle) = *actuals;

    let befores: Vec<_> = before_middle.iter().map(|flag| {
        derive_mode(flag.0.load(Ordering::Relaxed))
    }).collect();

    let handler = derive_mode(handler_middle.0.load(Ordering::Relaxed));

    let afters: Vec<_> = after_middle.iter().map(|flag| {
        derive_mode(flag.0.load(Ordering::Relaxed))
    }).collect();

    (befores, handler, afters)
}

fn test_chain(test_chain: ChainLike<Kind>, expected_chain: ChainLike<Kind>) {
    let actuals = counters(&test_chain);

    let (befores, handler, afters) = into_middle(test_chain, &actuals);

    let mut chain = Chain::new(handler);
    
    for middle in befores {
        chain.before(middle);
    }
    for middle in afters {
        chain.after(middle);
    }

    let _ = chain.handle(&mut request());

    let result = to_mode(&actuals);

    assert_eq!(result, expected_chain);
}

#[test]
fn test_chain_normal() {
    test_chain(
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Fine, Fine]),
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Fine, Fine]),
    )
}

#[test]
fn test_chain_before_error() {
    test_chain(
        (vec![Prob, Prob, Prob], Fine, vec![Prob, Prob, Prob]),
        (vec![Fine, Prob, Prob], Prob, vec![Prob, Prob, Prob])
    )
}

#[test]
fn test_chain_handler_error() {
    test_chain(
        (vec![Fine, Fine, Fine], Prob, vec![Prob, Prob, Prob]),
        (vec![Fine, Fine, Fine], Fine, vec![Prob, Prob, Prob])
    )
}

#[test]
fn test_chain_after_error() {
    test_chain(
        (vec![Fine, Fine, Fine], Fine, vec![Prob, Prob, Prob]),
        (vec![Fine, Fine, Fine], Fine, vec![Fine, Prob, Prob]),
    )
}

#[test]
fn test_chain_before_error_then_handle() {
    test_chain(
        (vec![Prob, Prob, Fine, Fine], Fine, vec![Fine]),
        (vec![Fine, Prob, Prob, Fine], Fine, vec![Fine]),
    )
}

#[test]
fn test_chain_after_error_then_handle() {
    test_chain(
        (vec![], Fine, vec![Prob, Prob, Fine, Fine]),
        (vec![], Fine, vec![Fine, Prob, Prob, Fine])
    )
}

#[test]
fn test_chain_handler_error_then_handle() {
    test_chain(
        (vec![], Prob, vec![Prob, Fine, Fine]),
        (vec![], Fine, vec![Prob, Prob, Fine])
    )
}
