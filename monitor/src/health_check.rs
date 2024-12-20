use std::future::Future;

pub enum Health {
    Unhealthy {
        reason: Option<String>,
    },

    Healthy,
}

// Something that can be
// healthy / unhealthy
pub trait Healthcheck {
    fn on_unhealthy(&self, func: impl Fn()) -> impl Future {
        async move {
            let Health::Healthy = self.health() else {
                func(); return;
            };

            while let Health::Healthy = self.health() {
                std::hint::spin_loop();
            }

            func();
        }
    }

    fn on_healthy(&self, func: impl Fn()) -> impl Future {
        async move {
            let Health::Unhealthy { .. } = self.health() else {
                func(); return;
            };

            while let Health::Unhealthy {
                ..
            } = self.health() {
                std::hint::spin_loop();
            }

            func();
        }
    }

    fn health(&self) -> Health;
}
