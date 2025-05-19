use crate::{EitherMut, EitherRef, IntoEither, Tree};

impl<T: IntoEither> Tree for [T; 1] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0"]
    }
}

impl<T: IntoEither> Tree for [T; 2] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1"]
    }
}

impl<T: IntoEither> Tree for [T; 3] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2"]
    }
}

impl<T: IntoEither> Tree for [T; 4] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            "3" => Some(self[3].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            "3" => Some(self[3].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3"]
    }
}

impl<T: IntoEither> Tree for [T; 5] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            "3" => Some(self[3].as_either_ref()),
            "4" => Some(self[4].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            "3" => Some(self[3].as_either_mut()),
            "4" => Some(self[4].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4"]
    }
}

impl<T: IntoEither> Tree for [T; 6] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            "3" => Some(self[3].as_either_ref()),
            "4" => Some(self[4].as_either_ref()),
            "5" => Some(self[5].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            "3" => Some(self[3].as_either_mut()),
            "4" => Some(self[4].as_either_mut()),
            "5" => Some(self[5].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5"]
    }
}

impl<T: IntoEither> Tree for [T; 7] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            "3" => Some(self[3].as_either_ref()),
            "4" => Some(self[4].as_either_ref()),
            "5" => Some(self[5].as_either_ref()),
            "6" => Some(self[6].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            "3" => Some(self[3].as_either_mut()),
            "4" => Some(self[4].as_either_mut()),
            "5" => Some(self[5].as_either_mut()),
            "6" => Some(self[6].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6"]
    }
}

impl<T: IntoEither> Tree for [T; 8] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            "3" => Some(self[3].as_either_ref()),
            "4" => Some(self[4].as_either_ref()),
            "5" => Some(self[5].as_either_ref()),
            "6" => Some(self[6].as_either_ref()),
            "7" => Some(self[7].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            "3" => Some(self[3].as_either_mut()),
            "4" => Some(self[4].as_either_mut()),
            "5" => Some(self[5].as_either_mut()),
            "6" => Some(self[6].as_either_mut()),
            "7" => Some(self[7].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7"]
    }
}

impl<T: IntoEither> Tree for [T; 9] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            "3" => Some(self[3].as_either_ref()),
            "4" => Some(self[4].as_either_ref()),
            "5" => Some(self[5].as_either_ref()),
            "6" => Some(self[6].as_either_ref()),
            "7" => Some(self[7].as_either_ref()),
            "8" => Some(self[8].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            "3" => Some(self[3].as_either_mut()),
            "4" => Some(self[4].as_either_mut()),
            "5" => Some(self[5].as_either_mut()),
            "6" => Some(self[6].as_either_mut()),
            "7" => Some(self[7].as_either_mut()),
            "8" => Some(self[8].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7", "8"]
    }
}

impl<T: IntoEither> Tree for [T; 10] {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_ref()),
            "1" | "y" => Some(self[1].as_either_ref()),
            "2" | "z" => Some(self[2].as_either_ref()),
            "3" => Some(self[3].as_either_ref()),
            "4" => Some(self[4].as_either_ref()),
            "5" => Some(self[5].as_either_ref()),
            "6" => Some(self[6].as_either_ref()),
            "7" => Some(self[7].as_either_ref()),
            "8" => Some(self[8].as_either_ref()),
            "9" => Some(self[9].as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self[0].as_either_mut()),
            "1" | "y" => Some(self[1].as_either_mut()),
            "2" | "z" => Some(self[2].as_either_mut()),
            "3" => Some(self[3].as_either_mut()),
            "4" => Some(self[4].as_either_mut()),
            "5" => Some(self[5].as_either_mut()),
            "6" => Some(self[6].as_either_mut()),
            "7" => Some(self[7].as_either_mut()),
            "8" => Some(self[8].as_either_mut()),
            "9" => Some(self[9].as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
    }
}
