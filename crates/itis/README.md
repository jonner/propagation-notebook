# ITIS database schema for toasty

Partial [toasty](https://github.com/tokio-rs/toasty) ORM model definitions for
the [ITIS taxonomy database](https://www.itis.gov/downloads/index.html).

This is broken out into a separate crate so that the library/application can
use toasty to query the ITIS database but not have the ITIS models incorporated
into the application database schema.

The libpropagation library uses the `toasty::models!(crate::*)` macro to
register the schema so when these types were defined in the library, the
migration scripts would automatically create database tables for them even
though they were only intended to be used for an external database.
