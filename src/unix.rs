////////////////////////////////////////////////////////////////////////////////
//                                                                            //
//  Copyright (c) 2015 Mohd Tarmizi Mohd Affandi                              //
//                                                                            //
//  Permission is hereby granted, free of charge, to any person obtaining a   //
//  copy of this software and associated documentation files (the             //
//  "Software"), to deal in the Software without restriction, including       //
//  without limitation the rights to use, copy, modify, merge, publish,       //
//  distribute, sublicense, and/or sell copies of the Software, and to        //
//  permit persons to whom the Software is furnished to do so, subject to     //
//  the following conditions:                                                 //
//                                                                            //
//  The above copyright notice and this permission notice shall be included   //
//  in all copies or substantial portions of the Software.                    //
//                                                                            //
//  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS   //
//  OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF                //
//  MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.    //
//  IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY      //
//  CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,      //
//  TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE         //
//  SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.                    //
//                                                                            //
////////////////////////////////////////////////////////////////////////////////

use nix::errno::Errno;
use nix::sys::socket;
use nix::sys::socket::SockAddr::*;
use nix::unistd;
use nix::Error::Sys;
use std::io::{self, Read, Write};
use std::os::unix::io::RawFd;

const LISTENSOCK_FILENO: RawFd = 0;

pub struct Transport {
    inner: RawFd,
}

impl Transport {
    pub fn new() -> Self {
        Self::from_raw_fd(LISTENSOCK_FILENO)
    }

    pub fn from_raw_fd(raw_fd: RawFd) -> Self {
        Transport { inner: raw_fd }
    }

    pub fn is_fastcgi(&self) -> bool {
        match socket::getpeername(self.inner) {
            Err(Sys(Errno::ENOTCONN)) => true,
            _ => false,
        }
    }

    pub fn accept(&mut self) -> io::Result<Socket> {
        match socket::accept(self.inner) {
            Ok(fd) => Ok(Socket { inner: fd }),
            Err(_) => Err(io::Error::last_os_error()),
        }
    }
}

pub struct Socket {
    inner: RawFd,
}

impl Socket {
    pub fn peer(&self) -> io::Result<String> {
        match socket::getpeername(self.inner) {
            Ok(Inet(addr)) => Ok(addr.to_str()),
            Ok(Unix(_)) => Ok("".into()),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported FastCGI socket",
            )),
        }
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        let _ = socket::shutdown(self.inner, socket::Shutdown::Write);
        let mut buf = Vec::new();
        self.read_to_end(&mut buf).ok();
        let _ = unistd::close(self.inner);
    }
}

impl<'a> Read for &'a Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match unistd::read(self.inner, buf) {
            Ok(size) => Ok(size),
            Err(_) => Err(io::Error::last_os_error()),
        }
    }
}

impl<'a> Write for &'a Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match unistd::write(self.inner, buf) {
            Ok(size) => Ok(size),
            Err(_) => Err(io::Error::last_os_error()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&*self).read(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&*self).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&*self).flush()
    }
}
