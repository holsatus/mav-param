use crate::{EitherMut, EitherRef, IntoEither, Tree};

impl<T0: IntoEither> Tree for (T0,) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0"]
    }
}

impl<T0: IntoEither, T1: IntoEither> Tree for (T0, T1) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1"]
    }
}

impl<T0: IntoEither, T1: IntoEither, T2: IntoEither> Tree for (T0, T1, T2) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2"]
    }
}

impl<T0: IntoEither, T1: IntoEither, T2: IntoEither, T3: IntoEither> Tree for (T0, T1, T2, T3) {
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            "3" => Some(self.3.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            "3" => Some(self.3.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3"]
    }
}

impl<T0: IntoEither, T1: IntoEither, T2: IntoEither, T3: IntoEither, T4: IntoEither> Tree
    for (T0, T1, T2, T3, T4)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            "3" => Some(self.3.as_either_ref()),
            "4" => Some(self.4.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            "3" => Some(self.3.as_either_mut()),
            "4" => Some(self.4.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4"]
    }
}

impl<T0: IntoEither, T1: IntoEither, T2: IntoEither, T3: IntoEither, T4: IntoEither, T5: IntoEither>
    Tree for (T0, T1, T2, T3, T4, T5)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            "3" => Some(self.3.as_either_ref()),
            "4" => Some(self.4.as_either_ref()),
            "5" => Some(self.5.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            "3" => Some(self.3.as_either_mut()),
            "4" => Some(self.4.as_either_mut()),
            "5" => Some(self.5.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5"]
    }
}

impl<
    T0: IntoEither,
    T1: IntoEither,
    T2: IntoEither,
    T3: IntoEither,
    T4: IntoEither,
    T5: IntoEither,
    T6: IntoEither,
> Tree for (T0, T1, T2, T3, T4, T5, T6)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            "3" => Some(self.3.as_either_ref()),
            "4" => Some(self.4.as_either_ref()),
            "5" => Some(self.5.as_either_ref()),
            "6" => Some(self.6.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            "3" => Some(self.3.as_either_mut()),
            "4" => Some(self.4.as_either_mut()),
            "5" => Some(self.5.as_either_mut()),
            "6" => Some(self.6.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6"]
    }
}

impl<
    T0: IntoEither,
    T1: IntoEither,
    T2: IntoEither,
    T3: IntoEither,
    T4: IntoEither,
    T5: IntoEither,
    T6: IntoEither,
    T7: IntoEither,
> Tree for (T0, T1, T2, T3, T4, T5, T6, T7)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            "3" => Some(self.3.as_either_ref()),
            "4" => Some(self.4.as_either_ref()),
            "5" => Some(self.5.as_either_ref()),
            "6" => Some(self.6.as_either_ref()),
            "7" => Some(self.7.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            "3" => Some(self.3.as_either_mut()),
            "4" => Some(self.4.as_either_mut()),
            "5" => Some(self.5.as_either_mut()),
            "6" => Some(self.6.as_either_mut()),
            "7" => Some(self.7.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7"]
    }
}

impl<
    T0: IntoEither,
    T1: IntoEither,
    T2: IntoEither,
    T3: IntoEither,
    T4: IntoEither,
    T5: IntoEither,
    T6: IntoEither,
    T7: IntoEither,
    T8: IntoEither,
> Tree for (T0, T1, T2, T3, T4, T5, T6, T7, T8)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            "3" => Some(self.3.as_either_ref()),
            "4" => Some(self.4.as_either_ref()),
            "5" => Some(self.5.as_either_ref()),
            "6" => Some(self.6.as_either_ref()),
            "7" => Some(self.7.as_either_ref()),
            "8" => Some(self.8.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            "3" => Some(self.3.as_either_mut()),
            "4" => Some(self.4.as_either_mut()),
            "5" => Some(self.5.as_either_mut()),
            "6" => Some(self.6.as_either_mut()),
            "7" => Some(self.7.as_either_mut()),
            "8" => Some(self.8.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7", "8"]
    }
}

impl<
    T0: IntoEither,
    T1: IntoEither,
    T2: IntoEither,
    T3: IntoEither,
    T4: IntoEither,
    T5: IntoEither,
    T6: IntoEither,
    T7: IntoEither,
    T8: IntoEither,
    T9: IntoEither,
> Tree for (T0, T1, T2, T3, T4, T5, T6, T7, T8, T9)
{
    fn get_ref<'a>(&'a self, path: &str) -> Option<EitherRef<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_ref()),
            "1" | "y" => Some(self.1.as_either_ref()),
            "2" | "z" => Some(self.2.as_either_ref()),
            "3" => Some(self.3.as_either_ref()),
            "4" => Some(self.4.as_either_ref()),
            "5" => Some(self.5.as_either_ref()),
            "6" => Some(self.6.as_either_ref()),
            "7" => Some(self.7.as_either_ref()),
            "8" => Some(self.8.as_either_ref()),
            "9" => Some(self.9.as_either_ref()),
            _ => None,
        }
    }

    fn get_mut<'a>(&'a mut self, path: &str) -> Option<EitherMut<'a>> {
        match path {
            "0" | "x" => Some(self.0.as_either_mut()),
            "1" | "y" => Some(self.1.as_either_mut()),
            "2" | "z" => Some(self.2.as_either_mut()),
            "3" => Some(self.3.as_either_mut()),
            "4" => Some(self.4.as_either_mut()),
            "5" => Some(self.5.as_either_mut()),
            "6" => Some(self.6.as_either_mut()),
            "7" => Some(self.7.as_either_mut()),
            "8" => Some(self.8.as_either_mut()),
            "9" => Some(self.9.as_either_mut()),
            _ => None,
        }
    }

    fn entries(&self) -> &'static [&'static str] {
        &["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"]
    }
}
