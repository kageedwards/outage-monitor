[![Build Status][build-shield]][build-url]
[![Issues][issues-shield]][issues-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![MIT License][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url]

<div align="center">
  <hr>
  <h3>Power Outage Monitor</h3>
  <h4>with Telegram alerts</h4>
  <hr>
</div>

_This project was designed for my own personal productivity needs and is by no means intended for wide distribution or production use as-is. Use or refactor for your own needs at your own discretion._

I created a CLI application to poll SCL power outage data and determine if a particular Seattle location is currently without power. Telegram alerts will be sent when power goes offline and when it returns.

Naturally, the outage monitor cannot run at the location being monitored.

------------------------------------------------------------------------------------

## Techniques and Skills I Applied

* Requesting real-world data from an API endpoint
* Typing API responses using generics
* Comparing geospatial data
* Using monadic patterns (Result, Option, Some, None) rather than try / catch in order to
    * Implement propagated, multi-threaded error handling
    * Propagate or handle an exception at the appropriate level
    * Account for all non-deterministic procedures (API responses, casts based on non-determistic values)
* Separating data analysis and data retrieval processes
* Using mutexes to protect application state and prevent data races
* Using Github Workflows to run build tests
* Demonstrating ownership over my role as assistant property manager by boosting observability
* Having the patience to over-engineer this just enough in Rust to demonstrate all the above in one project

[build-shield]: https://img.shields.io/github/actions/workflow/status/kageedwards/outage-monitor/rust.yml?style=for-the-badge
[build-url]: https://github.com/kageedwards/outage-monitor/actions
[forks-shield]: https://img.shields.io/github/forks/kageedwards/outage-monitor.svg?style=for-the-badge
[forks-url]: https://github.com/kageedwards/outage-monitor/network/members
[stars-shield]: https://img.shields.io/github/stars/kageedwards/outage-monitor.svg?style=for-the-badge
[stars-url]: https://github.com/kageedwards/outage-monitor/stargazers
[issues-shield]: https://img.shields.io/github/issues/kageedwards/outage-monitor.svg?style=for-the-badge
[issues-url]: https://github.com/kageedwards/outage-monitor/issues
[license-shield]: https://img.shields.io/github/license/kageedwards/outage-monitor.svg?style=for-the-badge
[license-url]: https://github.com/kageedwards/outage-monitor/blob/master/LICENSE
[linkedin-shield]: https://img.shields.io/badge/-LinkedIn-black.svg?style=for-the-badge&logo=linkedin&colorB=09f
[linkedin-url]: https://linkedin.com/in/kageedwards
