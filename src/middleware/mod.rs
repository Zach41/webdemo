use {WebError, WebResult, Request, Response};

pub trait Handler: Send + Sync + 'static {
    fn handle(&self, _: &mut Request) -> WebResult<Response>;
}

pub trait BeforeMiddleware: Send + Sync + 'static {
    fn before(&self, _: &mut Request) -> WebResult<()> { Ok(()) }

    fn catch(&self, _: &mut Request, err: WebError) -> WebResult<()> { Err(err) }
}

pub trait AfterMiddleware: Send + Sync + 'static {
    fn after(&self, _: &mut Request, resp: Response) -> WebResult<Response> { Ok(resp) }

    fn catch(&self, _: &mut Request, err: WebError) -> WebResult<Response> { Err(err) }
}

pub trait AroundMiddleware: Send + Sync + 'static {
    fn around(self, handler: Box<Handler>) -> Box<Handler>;
}

pub struct Chain {
    befores: Vec<Box<BeforeMiddleware>>,
    afters: Vec<Box<AfterMiddleware>>,
    handler: Option<Box<Handler>>,
}

impl Chain {
    pub fn new<H: Handler>(handler: H) -> Chain {
        Chain {
            befores: Vec::new(),
            afters: Vec::new(),
            handler: Some(Box::new(handler)),
        }
    }

    pub fn link<B, A>(&mut self, before: B, after: A) -> &mut Chain
        where B: BeforeMiddleware, A: AfterMiddleware {
        self.befores.push(Box::new(before));
        self.afters.push(Box::new(after));
        self
    }

    pub fn before<B: BeforeMiddleware>(&mut self, before: B) -> &mut Chain {
        self.befores.push(Box::new(before));
        self
    }

    pub fn after<A: AfterMiddleware>(&mut self, after: A) -> &mut Chain {
        self.afters.push(Box::new(after));
        self
    }
}

impl Handler for Chain {
    fn handle(&self, req: &mut Request) -> WebResult<Response> {
        self.continue_from_before(0, req)
    }
}

impl Chain {
    fn continue_from_before(&self, idx: usize, req: &mut Request) -> WebResult<Response> {
        info!("continue from before");
        if idx >= self.befores.len() {
            return self.continue_from_handler(req)
        }

        for i in idx .. self.befores.len() {
            match self.befores[i].as_ref().before(req) {
                Ok(()) => (),
                Err(err) => return self.fail_from_before(i + 1, req, err)
            }
        }
        self.continue_from_handler(req)
    }

    fn continue_from_after(&self, idx: usize, req: &mut Request, mut resp: Response) -> WebResult<Response> {
        info!("continue from after");
        if idx >= self.afters.len() {
            // done
            return Ok(resp)
        }

        for i in idx .. self.afters.len() {
            resp = match self.afters[i].as_ref().after(req, resp) {
                Ok(resp) => resp,
                Err(err) => return self.fail_from_after(i + 1, req, err)
            };            
        }
        Ok(resp)
    }

    fn continue_from_handler(&self, req: &mut Request) -> WebResult<Response> {
        info!("continue from handler");
        match self.handler.as_ref().unwrap().handle(req) {
            Ok(resp) => self.continue_from_after(0, req, resp),
            Err(err) => self.fail_from_after(0, req, err),
        }        
    }

    fn fail_from_before(&self, idx: usize, req: &mut Request,mut err: WebError) -> WebResult<Response> {
        info!("fail from before");
        if idx >= self.befores.len() {
            return self.fail_from_handler(req, err)
        }

        for i in idx .. self.befores.len() {
            err = match self.befores[i].as_ref().catch(req, err) {
                Ok(()) => return self.continue_from_before(i+ 1, req),
                Err(err) => err,
            };
        }
        self.fail_from_handler(req, err)
    }

    fn fail_from_after(&self, idx: usize, req: &mut Request, mut err: WebError) -> WebResult<Response> {
        info!("fail from after");
        if idx >= self.afters.len() {
            return Err(err)
        }

        for i in idx .. self.afters.len() {
            err = match self.afters[i].as_ref().catch(req, err) {
                Ok(resp) => return self.continue_from_after(i+ 1, req, resp),
                Err(err) => err,
            };
        }

        Err(err)
    }

    fn fail_from_handler(&self, req: &mut Request, err: WebError) -> WebResult<Response> {
        // TODO: use debug! macro
        info!("fail from handler");
        self.fail_from_after(0, req, err)
    }
}

impl<F> Handler for F
    where F: Fn(&mut Request) -> WebResult<Response> + Send + Sync + 'static {
    fn handle(&self, req: &mut Request) -> WebResult<Response> {
        (*self)(req)
    }
}

impl Handler for Box<Handler> {
    fn handle(&self, req: &mut Request) -> WebResult<Response> {
        (*self).as_ref().handle(req)
    }
}

impl<F> BeforeMiddleware for F
    where F: Fn(&mut Request) -> WebResult<()> + Send + Sync + 'static {
    fn before(&self, req: &mut Request) -> WebResult<()> {
        (*self)(req)
    }
}

impl BeforeMiddleware for Box<BeforeMiddleware> {
    fn before(&self, req: &mut Request) -> WebResult<()> {
        (*self).as_ref().before(req)
    }

    fn catch(&self, req: &mut Request, err: WebError) -> WebResult<()> {
        (*self).as_ref().catch(req, err)
    }
}

impl<F> AfterMiddleware for F
    where F: Send + Sync + 'static + Fn(*mut Request, Response) -> WebResult<Response> {
    fn after(&self, req: &mut Request, resp: Response) -> WebResult<Response> {
        (*self)(req, resp)
    }
}

impl AfterMiddleware for Box<AfterMiddleware> {
    fn after(&self, req: &mut Request, resp: Response) -> WebResult<Response> {
        (*self).as_ref().after(req, resp)
    }

    fn catch(&self, req: &mut Request, err: WebError) -> WebResult<Response> {
        (*self).as_ref().catch(req, err)
    }
}

impl<F> AroundMiddleware for F
    where F: Send + Sync + 'static + FnOnce(Box<Handler>) -> Box<Handler> {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
        self(handler)
    }
}

#[cfg(test)]
mod test;
