//! Russian texts of notifications: one line and a sentence for the feed, the same in letters,
//! plus the letter around them. Rendered on every read from the stored payload, so a wording fix
//! reaches old items too.

use chrono::{DateTime, FixedOffset, Utc};
use matching::trust::TrustLevel;

use super::{Audience, ComplaintAbout, Event, GuardianOutcome, NotificationKind, ReviewNote};
use crate::dto::{ApplicationStatus, DeclineReason, HiringStage};

/// Letters differ from the feed in one place: a company letter never quotes the candidate's own
/// words, which may hold anything, contacts included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    InApp,
    Email,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub title: String,
    pub body: String,
    /// Path in the cabinet.
    pub link: Option<String>,
}

const MONTHS: [&str; 12] = [
    "января",
    "февраля",
    "марта",
    "апреля",
    "мая",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октября",
    "ноября",
    "декабря",
];

fn moscow(at: DateTime<Utc>) -> DateTime<FixedOffset> {
    match FixedOffset::east_opt(3 * 3600) {
        Some(msk) => at.with_timezone(&msk),
        None => at.fixed_offset(),
    }
}

/// «20 октября» in Moscow time: the platform's dates are Moscow dates.
pub fn day(at: DateTime<Utc>) -> String {
    use chrono::Datelike;
    let m = moscow(at);
    format!("{} {}", m.day(), MONTHS[m.month0() as usize])
}

/// «20 октября, 14:30 мск».
pub fn day_time(at: DateTime<Utc>) -> String {
    format!("{}, {} мск", day(at), moscow(at).format("%H:%M"))
}

/// «280 000» with a no-break space between thousands, as the cabinet shows it.
pub fn rub(v: i64) -> String {
    let digits = v.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + 4);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\u{a0}');
        }
        out.push(c);
    }
    if v < 0 { format!("-{out}") } else { out }
}

pub fn salary(min: i64, max: i64) -> String {
    if min == max {
        format!("{}\u{a0}₽", rub(min))
    } else {
        format!("{} – {}\u{a0}₽", rub(min), rub(max))
    }
}

/// One of three Russian plural forms: 1 приглашение, 2 приглашения, 5 приглашений.
pub fn plural(n: usize, forms: [&'static str; 3]) -> &'static str {
    let (n10, n100) = (n % 10, n % 100);
    if n10 == 1 && n100 != 11 {
        forms[0]
    } else if (2..=4).contains(&n10) && !(12..=14).contains(&n100) {
        forms[1]
    } else {
        forms[2]
    }
}

fn stage_label(stage: HiringStage) -> &'static str {
    match stage {
        HiringStage::Interview => "собеседование",
        HiringStage::Offer => "оффер",
        HiringStage::Hired => "вы наняты",
        HiringStage::Rejected => "отказ",
    }
}

fn reason_label(reason: Option<DeclineReason>) -> &'static str {
    match reason {
        Some(DeclineReason::Salary) => "зарплата",
        Some(DeclineReason::Stack) => "стек",
        Some(DeclineReason::Format) => "формат работы",
        Some(DeclineReason::Company) => "компания",
        Some(DeclineReason::NotLooking) => "сейчас не ищет работу",
        Some(DeclineReason::Other) | None => "другое",
    }
}

pub fn trust_label(level: TrustLevel) -> &'static str {
    match level {
        TrustLevel::High => "Высокий уровень доверия",
        TrustLevel::Standard => "Обычный уровень доверия",
        TrustLevel::New => "Новая компания",
        TrustLevel::Low => "Квота временно снижена",
        TrustLevel::Frozen => "Приглашения приостановлены модератором",
    }
}

fn clip(s: &str, max: usize) -> String {
    let mut out: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        out.push('…');
    }
    out
}

/// The sentence around the quote ends with its own period: «…аннулирован». — not «…аннулирован.».
fn comment(c: &str) -> String {
    let c = c.trim();
    let c = c
        .strip_suffix('.')
        .filter(|r| !r.ends_with('.'))
        .unwrap_or(c);
    format!("«{}»", clip(c, 300))
}

fn category_now(n: &ReviewNote) -> String {
    match n.category_now {
        Some(c) => format!("Категория сейчас: {}.", c.name_ru()),
        None => "Категории сейчас нет.".to_owned(),
    }
}

fn invitations(quota: i16) -> String {
    let n = usize::try_from(quota).unwrap_or(0);
    format!(
        "{quota} {}",
        plural(n, ["приглашение", "приглашения", "приглашений"])
    )
}

fn upheld(n: u32) -> String {
    if n == 0 {
        String::new()
    } else {
        format!(" Подтверждено жалоб: {n} — они снижают квоту 90 дней.")
    }
}

pub fn text(event: &Event, channel: Channel) -> Text {
    match event {
        Event::InvitationReceived(n) => {
            let format = n
                .work_format
                .map(|f| format!(", {}", f.name_ru().to_lowercase()))
                .unwrap_or_default();
            Text {
                title: format!("{} приглашает: {}", n.company_name, n.title),
                body: format!(
                    "Вилка {}{format}. Ответить можно до {}.",
                    salary(n.salary_min, n.salary_max),
                    day(n.expires_at)
                ),
                link: Some(format!("/candidate/invitations/{}", n.invitation_id)),
            }
        }
        Event::InvitationExpiring(n) => Text {
            title: format!(
                "Приглашение от {} закроется через двое суток",
                n.company_name
            ),
            body: format!(
                "«{}», {}. Если не ответить до {}, приглашение закроется само.",
                n.title,
                salary(n.salary_min, n.salary_max),
                day_time(n.expires_at)
            ),
            link: Some(format!("/candidate/invitations/{}", n.invitation_id)),
        },
        Event::InvitationWithdrawn(n) => Text {
            title: format!("Компания {} отозвала приглашение", n.company_name),
            body: format!("«{}» больше не ждёт ответа.", n.title),
            link: Some(format!("/candidate/invitations/{}", n.invitation_id)),
        },
        Event::HiringStage(n) => {
            let reason = n
                .reason
                .as_deref()
                .map(|r| format!(" Комментарий компании: {}", clip(r, 300)))
                .unwrap_or_default();
            Text {
                title: format!(
                    "Компания {} отметила: {}",
                    n.company_name,
                    stage_label(n.stage)
                ),
                body: format!("«{}».{reason}", n.title),
                link: Some(match n.invitation_id {
                    Some(id) => format!("/candidate/invitations/{id}"),
                    None => "/candidate/vacancies".into(),
                }),
            }
        }
        Event::ApplicationStatus(n) => {
            let (title, tail) = match n.status {
                ApplicationStatus::Viewed => (
                    format!("Компания {} посмотрела ваш отклик", n.company_name),
                    "",
                ),
                ApplicationStatus::Accepted => (
                    format!("Компания {} приглашает по вашему отклику", n.company_name),
                    " Ваши контакты у компании уже есть — ждите связи.",
                ),
                ApplicationStatus::Rejected => (
                    format!("Компания {} отказала по вашему отклику", n.company_name),
                    " Другие вакансии и приглашения — в кабинете.",
                ),
                ApplicationStatus::Sent | ApplicationStatus::Withdrawn => {
                    (format!("Отклик в {}: статус изменился", n.company_name), "")
                }
            };
            Text {
                title,
                body: format!("Вакансия «{}».{tail}", n.vacancy_title),
                link: Some("/candidate/vacancies".into()),
            }
        }
        Event::AssessmentResult(n) => {
            let category = n.category.name_ru();
            let (title, body) = if n.passed {
                (
                    format!("Категория присвоена: {category}"),
                    "Тест пройден. Компании теперь находят вас в этой категории.".to_owned(),
                )
            } else {
                (
                    format!("Тест на {category} не пройден"),
                    "Категорию по этому тесту не присвоили. Какой тест доступен сейчас, видно на странице результата.".to_owned(),
                )
            };
            Text {
                title,
                body,
                link: Some(format!("/candidate/assessment/{}", n.session_id)),
            }
        }
        Event::GuardianDecision(n) => {
            let (title, body) = match n.decision {
                GuardianOutcome::Granted => (
                    "Законный представитель дал согласие",
                    "Теперь можно показывать профиль компаниям, принимать приглашения и откликаться на вакансии.",
                ),
                GuardianOutcome::Declined => (
                    "Законный представитель не дал согласие",
                    "Профиль скрыт от компаний. Отправить письмо ещё раз или указать другой адрес можно на главной кабинета.",
                ),
                GuardianOutcome::Revoked => (
                    "Законный представитель отозвал согласие",
                    "Профиль скрыт от компаний. Попросить согласие снова можно на главной кабинета.",
                ),
            };
            Text {
                title: title.into(),
                body: body.into(),
                link: Some("/candidate".into()),
            }
        }
        Event::InvitationAccepted(n) => Text {
            title: format!(
                "Кандидат {} принял приглашение — контакты в кабинете",
                n.candidate_code
            ),
            body: match channel {
                Channel::InApp => format!(
                    "«{}». Имя, телефон и почта — в карточке кандидата.",
                    n.title
                ),
                Channel::Email => format!(
                    "«{}». Имя, телефон и почта — в карточке кандидата: в письмах мы их не пересылаем.",
                    n.title
                ),
            },
            link: Some(format!("/employer/candidates/{}", n.candidate_id)),
        },
        Event::InvitationDeclined(n) => {
            let comment = match (channel, n.comment.as_deref()) {
                (Channel::InApp, Some(c)) => format!(" Комментарий: «{}»", clip(c, 300)),
                (Channel::Email, Some(_)) => {
                    " Кандидат оставил комментарий — он в кабинете.".into()
                }
                (_, None) => String::new(),
            };
            Text {
                title: format!("Кандидат {} отклонил приглашение", n.candidate_code),
                body: format!(
                    "«{}». Причина: {}.{comment}",
                    n.title,
                    reason_label(n.reason)
                ),
                link: Some("/employer/invitations".into()),
            }
        }
        Event::ApplicationReceived(n) => Text {
            title: format!("Новый отклик на «{}»", n.vacancy_title),
            body: format!(
                "Кандидат {}. Контакты и сопроводительное письмо — в кабинете.",
                n.candidate_code
            ),
            link: Some(format!("/employer/vacancies/{}", n.vacancy_id)),
        },
        Event::WebhookFailing(n) => Text {
            title: "ATS компании не принимает события".into(),
            body: format!(
                "Событие {} не удалось доставить {} {} подряд, последняя ошибка: {}. Повторы идут сами — проверьте адрес вебхука в разделе «Интеграции».",
                n.event_type,
                n.failed_attempts,
                plural(
                    usize::try_from(n.failed_attempts).unwrap_or(0),
                    ["раз", "раза", "раз"]
                ),
                clip(&n.error, 200)
            ),
            link: Some("/employer/integrations".into()),
        },
        Event::TrustChanged(n) => {
            let title = if n.quota != n.quota_before {
                format!(
                    "Квота приглашений: {} → {} за 7 дней",
                    n.quota_before, n.quota
                )
            } else {
                format!("Уровень доверия: {}", trust_label(n.level).to_lowercase())
            };
            Text {
                title,
                body: format!(
                    "{}. Что влияет на квоту — в разделе «Доверие и квота».",
                    trust_label(n.level)
                ),
                link: Some("/employer/trust".into()),
            }
        }
        Event::ResultConfirmed(n) => Text {
            title: format!(
                "Модератор подтвердил результат теста: {}",
                n.category.name_ru()
            ),
            body: format!(
                "{} Комментарий модератора: {}.",
                if n.badge {
                    "Отметка «требует подтверждения» снята, компании видят «Результат подтверждён»."
                } else {
                    "Отметка «требует подтверждения» снята."
                },
                comment(&n.comment)
            ),
            link: Some(format!("/candidate/assessment/{}", n.session_id)),
        },
        Event::ResultAnnulled(n) => Text {
            title: format!("Результат теста аннулирован: {}", n.category.name_ru()),
            body: format!(
                "Комментарий модератора: {}. {} Тест на {} снова можно пройти через 30 дней.",
                comment(&n.comment),
                category_now(n),
                n.category.grade.name_ru()
            ),
            link: Some(format!("/candidate/assessment/{}", n.session_id)),
        },
        Event::RetestOrdered(n) => Text {
            title: format!(
                "Назначена пересдача с видеозаписью: {}",
                n.category.name_ru()
            ),
            body: format!(
                "Можно пройти сразу, без ожидания, но только с включённой камерой. Комментарий модератора: {}. {}",
                comment(&n.comment),
                category_now(n)
            ),
            link: Some(format!("/candidate/assessment/{}", n.session_id)),
        },
        Event::VacancyUnpublished(n) => Text {
            title: format!("Вакансия «{}» снята с публикации", n.title),
            body: format!(
                "Комментарий модератора: {}. Вернуть вакансию может только модератор.{}",
                comment(&n.comment),
                upheld(n.complaints_upheld)
            ),
            link: Some(format!("/employer/vacancies/{}", n.vacancy_id)),
        },
        Event::InvitationsFrozen(n) => Text {
            title: format!("Приглашения приостановлены до {}", day(n.until)),
            body: format!(
                "До {} квота приглашений — 0. Комментарий модератора: {}.{}",
                day_time(n.until),
                comment(&n.comment),
                upheld(n.complaints_upheld)
            ),
            link: Some("/employer/trust".into()),
        },
        Event::SanctionLifted(n) => match (n.vacancy_id, n.title.as_deref()) {
            (Some(id), title) => Text {
                title: format!("Вакансия «{}» снова опубликована", title.unwrap_or("")),
                body: format!("Модератор отменил снятие: {}.", comment(&n.comment)),
                link: Some(format!("/employer/vacancies/{id}")),
            },
            (None, _) => Text {
                title: "Приглашения снова доступны".into(),
                body: format!(
                    "Модератор снял приостановку досрочно: {}. Квота — {} за 7 дней.",
                    comment(&n.comment),
                    invitations(n.quota)
                ),
                link: Some("/employer/trust".into()),
            },
        },
        Event::ComplaintResolved(n) => Text {
            title: match (n.about, n.vacancy_title.as_deref()) {
                (ComplaintAbout::Vacancy, Some(t)) => {
                    format!("Жалоба на вакансию «{t}» не подтвердилась")
                }
                (ComplaintAbout::Vacancy, None) => "Жалоба на вакансию не подтвердилась".into(),
                (ComplaintAbout::Invitation, _) => "Жалоба на приглашение не подтвердилась".into(),
                (ComplaintAbout::Company, _) => "Жалоба на компанию не подтвердилась".into(),
            },
            body: format!(
                "Модератор проверил жалобу: нарушений нет, квоту она больше не снижает. Квота — {} за 7 дней.",
                invitations(n.quota)
            ),
            link: Some("/employer/trust".into()),
        },
        Event::FspAchievements(n) => {
            let title = if n.added > 0 {
                "Новые достижения ФСП в профиле"
            } else {
                "Данные ФСП в профиле обновились"
            };
            let what = match (n.added, n.latest.as_deref()) {
                (0, _) if n.removed > 0 => format!(
                    "Реестр ФСП убрал {} {}.",
                    n.removed,
                    plural(usize::from(n.removed), ["запись", "записи", "записей"])
                ),
                (0, _) => "Реестр ФСП уточнил записи о достижениях.".to_owned(),
                (1, Some(latest)) => format!("{latest}."),
                (k, Some(latest)) => format!("{latest} и ещё {}.", k - 1),
                (k, None) => format!(
                    "Реестр ФСП прислал {k} {}.",
                    plural(usize::from(k), ["результат", "результата", "результатов"])
                ),
            };
            Text {
                title: title.into(),
                body: format!("{what} ФСП-скор и навыки, подтверждённые ФСП, уже пересчитаны."),
                link: Some("/candidate/settings#fsp".into()),
            }
        }
    }
}

pub fn feed_path(audience: Audience) -> &'static str {
    match audience {
        Audience::Candidate => "/candidate/notifications",
        Audience::Employer => "/employer/notifications",
    }
}

/// Where the email switches are: candidate settings, the company page for an employer.
pub fn settings_path(audience: Audience) -> &'static str {
    match audience {
        Audience::Candidate => "/candidate/settings#notifications",
        Audience::Employer => "/employer/company#notifications",
    }
}

pub struct LetterContext<'a> {
    pub public_url: &'a str,
    pub audience: Audience,
    /// The page behind the letter's unsubscribe link.
    pub unsubscribe_url: &'a str,
    /// What the link turns off: a kind's title or «все письма».
    pub unsubscribe_label: &'a str,
}

#[derive(Debug, Clone)]
pub struct Letter {
    pub subject: String,
    pub text: String,
    pub html: String,
}

/// A letter leads with its newest invitation: the subject names it, the body starts with it. The
/// rest keep the order they came in.
fn ordered(items: &[Event]) -> Vec<&Event> {
    let invitation = |e: &&Event| e.kind() == NotificationKind::InvitationReceived;
    let mut out: Vec<&Event> = items.iter().rev().filter(invitation).collect();
    out.extend(items.iter().filter(|e| !invitation(e)));
    out
}

fn subject(items: &[&Event]) -> String {
    let n = items.len();
    match items {
        [] => String::new(),
        [one] => text(one, Channel::Email).title,
        [lead, rest @ ..] if lead.kind() == NotificationKind::InvitationReceived => {
            let more = rest.len();
            let forms = if rest
                .iter()
                .all(|e| e.kind() == NotificationKind::InvitationReceived)
            {
                ["приглашение", "приглашения", "приглашений"]
            } else {
                ["уведомление", "уведомления", "уведомлений"]
            };
            format!(
                "{} (и ещё {more} {})",
                text(lead, Channel::Email).title,
                plural(more, forms)
            )
        }
        _ => format!(
            "{n} {}",
            plural(
                n,
                [
                    "новое уведомление",
                    "новых уведомления",
                    "новых уведомлений"
                ]
            )
        ),
    }
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Plain text and simple HTML with the same content. Every item links into the cabinet; the
/// footer links to all notifications, the email settings and the one-click unsubscribe page.
pub fn letter(items: &[Event], ctx: &LetterContext<'_>) -> Letter {
    let base = ctx.public_url;
    let feed = format!("{base}{}", feed_path(ctx.audience));
    let settings = format!("{base}{}", settings_path(ctx.audience));
    let items = ordered(items);
    let texts: Vec<Text> = items.iter().map(|e| text(e, Channel::Email)).collect();

    let mut plain = String::from("Здравствуйте!\n\n");
    for t in &texts {
        plain.push_str(&t.title);
        plain.push('\n');
        plain.push_str(&t.body);
        plain.push('\n');
        if let Some(link) = &t.link {
            plain.push_str(&format!("{base}{link}\n"));
        }
        plain.push('\n');
    }
    plain.push_str(&format!(
        "Все уведомления: {feed}\nНастроить письма: {settings}\nОтписаться от писем «{}»: {}\n\n— ФСП Таланты\n",
        ctx.unsubscribe_label, ctx.unsubscribe_url
    ));

    let subject = subject(&items);
    let mut html = format!(
        "<!doctype html><html lang=\"ru\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title></head>\
         <body style=\"margin:0;padding:24px 12px;background:#f6f6f7;font-family:-apple-system,'Segoe UI',Roboto,Arial,sans-serif;color:#1b1c21\">\
         <div style=\"max-width:560px;margin:0 auto;background:#ffffff;border:1px solid #e4e4e8;border-radius:14px;padding:24px\">\
         <p style=\"margin:0 0 8px;font-size:15px\">Здравствуйте!</p>",
        escape(&subject)
    );
    for t in &texts {
        html.push_str(&format!(
            "<div style=\"border-top:1px solid #e4e4e8;padding:16px 0\">\
             <p style=\"margin:0;font-size:16px;font-weight:600\">{}</p>\
             <p style=\"margin:6px 0 0;font-size:15px;color:#5a5c66;line-height:1.5\">{}</p>",
            escape(&t.title),
            escape(&t.body)
        ));
        if let Some(link) = &t.link {
            html.push_str(&format!(
                "<p style=\"margin:12px 0 0\"><a href=\"{}\" style=\"display:inline-block;background:#402fff;color:#ffffff;text-decoration:none;border-radius:10px;padding:10px 16px;font-size:14px;font-weight:600\">Открыть в кабинете</a></p>",
                escape(&format!("{base}{link}"))
            ));
        }
        html.push_str("</div>");
    }
    html.push_str(&format!(
        "<p style=\"margin:16px 0 0;padding-top:16px;border-top:1px solid #e4e4e8;font-size:13px;color:#6b6e78;line-height:1.6\">\
         <a href=\"{}\" style=\"color:#402fff\">Все уведомления</a> · \
         <a href=\"{}\" style=\"color:#402fff\">Настроить письма</a> · \
         <a href=\"{}\" style=\"color:#6b6e78\">Отписаться от писем «{}»</a></p>\
         <p style=\"margin:12px 0 0;font-size:13px;color:#6b6e78\">— ФСП Таланты</p></div></body></html>",
        escape(&feed),
        escape(&settings),
        escape(ctx.unsubscribe_url),
        escape(ctx.unsubscribe_label)
    ));
    Letter {
        subject,
        text: plain,
        html,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notify::{
        AcceptedNote, ComplaintNote, DeclinedNote, FrozenNote, FspNote, InvitationNote, LiftedNote,
        TrustNote, UnpublishedNote,
    };
    use catalog::{Category, Grade, Specialization, WorkFormat};
    use uuid::Uuid;

    fn invitation(company: &str) -> Event {
        Event::InvitationReceived(InvitationNote {
            invitation_id: Uuid::nil(),
            company_name: company.into(),
            title: "Backend-разработчик".into(),
            salary_min: 280_000,
            salary_max: 340_000,
            work_format: Some(WorkFormat::Remote),
            expires_at: DateTime::parse_from_rfc3339("2026-10-20T09:12:44Z")
                .unwrap()
                .to_utc(),
        })
    }

    fn ctx() -> LetterContext<'static> {
        LetterContext {
            public_url: "https://talents.example",
            audience: Audience::Candidate,
            unsubscribe_url: "https://talents.example/notifications/unsubscribe?token=T",
            unsubscribe_label: "Новые приглашения",
        }
    }

    #[test]
    fn invitation_names_company_role_salary_and_deadline() {
        let t = text(&invitation("Рога и копыта"), Channel::Email);
        assert_eq!(t.title, "Рога и копыта приглашает: Backend-разработчик");
        assert_eq!(
            t.body,
            "Вилка 280\u{a0}000 – 340\u{a0}000\u{a0}₽, удалённо. Ответить можно до 20 октября."
        );
        assert_eq!(
            t.link.as_deref(),
            Some("/candidate/invitations/00000000-0000-0000-0000-000000000000")
        );
    }

    #[test]
    fn numbers_and_plurals() {
        assert_eq!(rub(5_000), "5\u{a0}000");
        assert_eq!(rub(999), "999");
        assert_eq!(rub(1_234_567), "1\u{a0}234\u{a0}567");
        let f = ["приглашение", "приглашения", "приглашений"];
        assert_eq!(
            [1, 2, 5, 11, 21, 22, 112].map(|n| plural(n, f)),
            [f[0], f[1], f[2], f[2], f[0], f[1], f[2]]
        );
        assert_eq!(
            day_time(
                DateTime::parse_from_rfc3339("2026-12-31T21:30:00Z")
                    .unwrap()
                    .to_utc()
            ),
            "1 января, 00:30 мск"
        );
    }

    #[test]
    fn several_invitations_become_one_letter() {
        let items = [invitation("Альфа"), invitation("Бета"), invitation("Гамма")];
        let letter = letter(&items, &ctx());
        assert_eq!(
            letter.subject,
            "Гамма приглашает: Backend-разработчик (и ещё 2 приглашения)"
        );
        assert!(
            letter.text.find("Гамма").unwrap() < letter.text.find("Альфа").unwrap(),
            "the newest invitation leads the body too"
        );
        for name in ["Альфа", "Бета", "Гамма"] {
            assert!(letter.text.contains(name) && letter.html.contains(name));
        }
        assert!(
            letter
                .text
                .contains("https://talents.example/candidate/notifications")
        );
        assert!(
            letter
                .text
                .contains("https://talents.example/candidate/settings#notifications")
        );
        assert!(letter.text.contains("Отписаться от писем «Новые приглашения»: https://talents.example/notifications/unsubscribe?token=T"));
    }

    #[test]
    fn html_escapes_what_users_typed() {
        let letter = letter(&[invitation("<script>alert(1)</script> & Co")], &ctx());
        assert!(!letter.html.contains("<script>"));
        assert!(
            letter
                .html
                .contains("&lt;script&gt;alert(1)&lt;/script&gt; &amp; Co")
        );
    }

    #[test]
    fn company_letter_carries_the_code_not_the_person() {
        let accepted = Event::InvitationAccepted(AcceptedNote {
            invitation_id: Uuid::nil(),
            candidate_id: Uuid::nil(),
            candidate_code: "K-7F3A2C".into(),
            title: "Backend".into(),
        });
        let declined = Event::InvitationDeclined(DeclinedNote {
            invitation_id: Uuid::nil(),
            candidate_code: "K-7F3A2C".into(),
            title: "Backend".into(),
            reason: Some(DeclineReason::Salary),
            comment: Some("Анна, +7 900 123-45-67, anna@example.ru".into()),
        });
        let ctx = LetterContext {
            audience: Audience::Employer,
            ..ctx()
        };
        let letter = letter(&[accepted.clone(), declined.clone()], &ctx);
        assert!(
            letter
                .text
                .contains("Кандидат K-7F3A2C принял приглашение — контакты в кабинете")
        );
        assert!(letter.text.contains("Причина: зарплата"));
        for leak in ["Анна", "+7 900", "anna@example.ru"] {
            assert!(
                !letter.text.contains(leak) && !letter.html.contains(leak),
                "{leak}"
            );
        }
        assert!(
            text(&declined, Channel::InApp)
                .body
                .contains("anna@example.ru"),
            "the cabinet shows what the company already sees there"
        );
        assert_eq!(subject(&[&accepted, &declined]), "2 новых уведомления");
    }

    #[test]
    fn a_mixed_letter_leads_with_the_newest_invitation() {
        let result = Event::FspAchievements(FspNote {
            added: 1,
            removed: 0,
            latest: Some("Призёр".into()),
        });
        let items = [invitation("Альфа"), result, invitation("Бета")];
        let lead = ordered(&items);
        assert_eq!(
            subject(&lead),
            "Бета приглашает: Backend-разработчик (и ещё 2 уведомления)"
        );
        assert_eq!(
            subject(&lead[..2]),
            "Бета приглашает: Backend-разработчик (и ещё 1 приглашение)"
        );
        let l = letter(&items, &ctx());
        let (beta, alpha, fsp) = (
            l.text.find("Бета").unwrap(),
            l.text.find("Альфа").unwrap(),
            l.text.find("Новые достижения ФСП").unwrap(),
        );
        assert!(beta < alpha && alpha < fsp, "{}", l.text);
        assert_eq!(
            subject(&ordered(&[items[1].clone(), invitation("Гамма")])),
            "Гамма приглашает: Backend-разработчик (и ещё 1 уведомление)"
        );
    }

    #[test]
    fn a_complaint_names_a_public_vacancy_but_not_an_invitation() {
        let about = |about, title: Option<&str>| {
            text(
                &Event::ComplaintResolved(ComplaintNote {
                    about,
                    vacancy_id: title.map(|_| Uuid::nil()),
                    vacancy_title: title.map(str::to_owned),
                    quota: 21,
                }),
                Channel::Email,
            )
        };
        let t = about(ComplaintAbout::Vacancy, Some("Go-разработчик"));
        assert_eq!(
            t.title,
            "Жалоба на вакансию «Go-разработчик» не подтвердилась"
        );
        assert!(
            t.body.ends_with("Квота — 21 приглашение за 7 дней."),
            "{}",
            t.body
        );
        let t = about(ComplaintAbout::Invitation, None);
        assert_eq!(t.title, "Жалоба на приглашение не подтвердилась");
        let lifted = text(
            &Event::SanctionLifted(LiftedNote {
                vacancy_id: None,
                title: None,
                comment: "  Проверка закончена  ".into(),
                quota: 25,
            }),
            Channel::InApp,
        );
        assert_eq!(
            lifted.body,
            "Модератор снял приостановку досрочно: «Проверка закончена». Квота — 25 приглашений за 7 дней."
        );
    }

    #[test]
    fn fsp_note_says_what_the_registry_changed() {
        let fsp = |added, removed, latest: Option<&str>| {
            text(
                &Event::FspAchievements(FspNote {
                    added,
                    removed,
                    latest: latest.map(str::to_owned),
                }),
                Channel::Email,
            )
        };
        let one = fsp(
            1,
            0,
            Some("Призёр (2 место) всероссийских соревнований, 2026"),
        );
        assert_eq!(one.title, "Новые достижения ФСП в профиле");
        assert_eq!(
            one.body,
            "Призёр (2 место) всероссийских соревнований, 2026. ФСП-скор и навыки, подтверждённые ФСП, уже пересчитаны."
        );
        assert!(fsp(3, 0, Some("X")).body.starts_with("X и ещё 2."));
        let gone = fsp(0, 2, None);
        assert_eq!(gone.title, "Данные ФСП в профиле обновились");
        assert!(gone.body.starts_with("Реестр ФСП убрал 2 записи."));
        assert_eq!(one.link.as_deref(), Some("/candidate/settings#fsp"));
    }

    const NIL_INVITATION: &str = "/candidate/invitations/00000000-0000-0000-0000-000000000000";
    const NIL_SESSION: &str = "/candidate/assessment/00000000-0000-0000-0000-000000000000";
    const BACKEND_MIDDLE: Category = Category::new(Specialization::Backend, Grade::Middle);

    fn with_kind(event: Event, kind: NotificationKind) -> Event {
        let Event::InvitationReceived(note) = event else {
            panic!("an invitation")
        };
        match kind {
            NotificationKind::InvitationExpiring => Event::InvitationExpiring(note),
            NotificationKind::InvitationWithdrawn => Event::InvitationWithdrawn(note),
            _ => Event::InvitationReceived(note),
        }
    }

    #[test]
    fn expiry_reminder_names_the_company_salary_and_the_exact_deadline() {
        let t = text(
            &with_kind(
                invitation("Рога и копыта"),
                NotificationKind::InvitationExpiring,
            ),
            Channel::Email,
        );
        assert_eq!(
            t.title,
            "Приглашение от Рога и копыта закроется через двое суток"
        );
        assert_eq!(
            t.body,
            "«Backend-разработчик», 280\u{a0}000 – 340\u{a0}000\u{a0}₽. Если не ответить до 20 октября, 12:12 мск, приглашение закроется само."
        );
        assert_eq!(t.link.as_deref(), Some(NIL_INVITATION));
    }

    #[test]
    fn withdrawn_invitation_says_it_no_longer_waits_for_an_answer() {
        let t = text(
            &with_kind(
                invitation("Рога и копыта"),
                NotificationKind::InvitationWithdrawn,
            ),
            Channel::InApp,
        );
        assert_eq!(t.title, "Компания Рога и копыта отозвала приглашение");
        assert_eq!(t.body, "«Backend-разработчик» больше не ждёт ответа.");
        assert_eq!(t.link.as_deref(), Some(NIL_INVITATION));
    }

    fn trust(level: TrustLevel, quota_before: i16, quota: i16) -> Text {
        text(
            &Event::TrustChanged(TrustNote {
                level_before: TrustLevel::New,
                level,
                quota_before,
                quota,
            }),
            Channel::Email,
        )
    }

    #[test]
    fn trust_change_leads_with_the_quota_when_it_moved() {
        let t = trust(TrustLevel::New, 10, 15);
        assert_eq!(t.title, "Квота приглашений: 10 → 15 за 7 дней");
        assert_eq!(
            t.body,
            "Новая компания. Что влияет на квоту — в разделе «Доверие и квота»."
        );
        assert_eq!(t.link.as_deref(), Some("/employer/trust"));
    }

    #[test]
    fn trust_change_names_the_level_when_the_quota_stayed() {
        let t = trust(TrustLevel::Low, 10, 10);
        assert_eq!(t.title, "Уровень доверия: квота временно снижена");
        assert_eq!(
            t.body,
            "Квота временно снижена. Что влияет на квоту — в разделе «Доверие и квота»."
        );
    }

    #[test]
    fn accepted_letter_links_to_the_candidate_card_in_the_cabinet() {
        let candidate = Uuid::from_u128(0x7f3a2c);
        let accepted = Event::InvitationAccepted(AcceptedNote {
            invitation_id: Uuid::nil(),
            candidate_id: candidate,
            candidate_code: "K-7F3A2C".into(),
            title: "Backend".into(),
        });
        let l = letter(
            &[accepted],
            &LetterContext {
                audience: Audience::Employer,
                ..ctx()
            },
        );
        assert_eq!(
            l.subject,
            "Кандидат K-7F3A2C принял приглашение — контакты в кабинете"
        );
        assert!(
            l.text.contains(&format!(
                "https://talents.example/employer/candidates/{candidate}"
            )),
            "{}",
            l.text
        );
        assert!(l.text.contains("в письмах мы их не пересылаем"));
        assert!(
            l.text
                .contains("https://talents.example/employer/company#notifications")
        );
    }

    fn review(comment: &str, category_now: Option<Category>, badge: bool) -> ReviewNote {
        ReviewNote {
            session_id: Uuid::nil(),
            category: BACKEND_MIDDLE,
            comment: comment.into(),
            category_now,
            badge,
        }
    }

    #[test]
    fn confirmed_result_says_the_mark_is_gone_and_quotes_the_moderator() {
        let t = text(
            &Event::ResultConfirmed(review(
                "Ответы совпадают с записью экрана.",
                Some(BACKEND_MIDDLE),
                false,
            )),
            Channel::Email,
        );
        assert_eq!(
            t.title,
            "Модератор подтвердил результат теста: Бэкенд-разработка · Middle"
        );
        assert_eq!(
            t.body,
            "Отметка «требует подтверждения» снята. Комментарий модератора: «Ответы совпадают с записью экрана»."
        );
        assert_eq!(t.link.as_deref(), Some(NIL_SESSION));
        let badge = text(
            &Event::ResultConfirmed(review("Всё чисто", Some(BACKEND_MIDDLE), true)),
            Channel::InApp,
        );
        assert_eq!(
            badge.body,
            "Отметка «требует подтверждения» снята, компании видят «Результат подтверждён». Комментарий модератора: «Всё чисто»."
        );
    }

    #[test]
    fn annulled_result_says_what_category_is_left() {
        let t = text(
            &Event::ResultAnnulled(review(
                "Ответы совпадают с утёкшим ключом",
                Some(Category::new(Specialization::Backend, Grade::Junior)),
                false,
            )),
            Channel::Email,
        );
        assert_eq!(
            t.title,
            "Результат теста аннулирован: Бэкенд-разработка · Middle"
        );
        assert_eq!(
            t.body,
            "Комментарий модератора: «Ответы совпадают с утёкшим ключом». Категория сейчас: Бэкенд-разработка · Junior. Тест на Middle снова можно пройти через 30 дней."
        );
        assert_eq!(t.link.as_deref(), Some(NIL_SESSION));
        let none = text(
            &Event::ResultAnnulled(review("Подсказки со второго экрана", None, false)),
            Channel::Email,
        );
        assert_eq!(
            none.body,
            "Комментарий модератора: «Подсказки со второго экрана». Категории сейчас нет. Тест на Middle снова можно пройти через 30 дней."
        );
    }

    #[test]
    fn retest_order_allows_an_immediate_attempt_with_the_camera_on() {
        let t = text(
            &Event::RetestOrdered(review("Камера была выключена", None, false)),
            Channel::Email,
        );
        assert_eq!(
            t.title,
            "Назначена пересдача с видеозаписью: Бэкенд-разработка · Middle"
        );
        assert_eq!(
            t.body,
            "Можно пройти сразу, без ожидания, но только с включённой камерой. Комментарий модератора: «Камера была выключена». Категории сейчас нет."
        );
        assert_eq!(t.link.as_deref(), Some(NIL_SESSION));
    }

    #[test]
    fn dismissed_complaint_about_the_company_or_an_untitled_vacancy() {
        let about = |about, vacancy_id| {
            text(
                &Event::ComplaintResolved(ComplaintNote {
                    about,
                    vacancy_id,
                    vacancy_title: None,
                    quota: 15,
                }),
                Channel::Email,
            )
        };
        let company = about(ComplaintAbout::Company, None);
        assert_eq!(company.title, "Жалоба на компанию не подтвердилась");
        assert_eq!(
            company.body,
            "Модератор проверил жалобу: нарушений нет, квоту она больше не снижает. Квота — 15 приглашений за 7 дней."
        );
        assert_eq!(company.link.as_deref(), Some("/employer/trust"));
        assert_eq!(
            about(ComplaintAbout::Vacancy, Some(Uuid::nil())).title,
            "Жалоба на вакансию не подтвердилась"
        );
        assert_eq!(
            about(ComplaintAbout::Invitation, None).link.as_deref(),
            Some("/employer/trust")
        );
    }

    #[test]
    fn unpublished_vacancy_quotes_the_moderator_and_counts_upheld_complaints() {
        let id = Uuid::from_u128(5);
        let unpublished = |complaints_upheld| {
            text(
                &Event::VacancyUnpublished(UnpublishedNote {
                    vacancy_id: id,
                    title: "Go-разработчик".into(),
                    comment: "Вилка не соответствует описанию.".into(),
                    complaints_upheld,
                }),
                Channel::Email,
            )
        };
        let t = unpublished(2);
        assert_eq!(t.title, "Вакансия «Go-разработчик» снята с публикации");
        assert_eq!(
            t.body,
            "Комментарий модератора: «Вилка не соответствует описанию». Вернуть вакансию может только модератор. Подтверждено жалоб: 2 — они снижают квоту 90 дней."
        );
        assert_eq!(t.link, Some(format!("/employer/vacancies/{id}")));
        assert_eq!(
            unpublished(0).body,
            "Комментарий модератора: «Вилка не соответствует описанию». Вернуть вакансию может только модератор."
        );
    }

    #[test]
    fn frozen_invitations_name_the_moment_they_return() {
        let t = text(
            &Event::InvitationsFrozen(FrozenNote {
                until: DateTime::parse_from_rfc3339("2026-10-20T09:12:44Z")
                    .unwrap()
                    .to_utc(),
                comment: "Массовые приглашения без отбора".into(),
                complaints_upheld: 3,
            }),
            Channel::Email,
        );
        assert_eq!(t.title, "Приглашения приостановлены до 20 октября");
        assert_eq!(
            t.body,
            "До 20 октября, 12:12 мск квота приглашений — 0. Комментарий модератора: «Массовые приглашения без отбора». Подтверждено жалоб: 3 — они снижают квоту 90 дней."
        );
        assert_eq!(t.link.as_deref(), Some("/employer/trust"));
    }

    #[test]
    fn lifted_sanction_names_the_vacancy_or_the_returned_invitations() {
        let id = Uuid::from_u128(5);
        let lifted = |vacancy_id: Option<Uuid>, title: Option<&str>| {
            text(
                &Event::SanctionLifted(LiftedNote {
                    vacancy_id,
                    title: title.map(str::to_owned),
                    comment: "Ошибка модерации".into(),
                    quota: 20,
                }),
                Channel::Email,
            )
        };
        let vacancy = lifted(Some(id), Some("Go-разработчик"));
        assert_eq!(
            vacancy.title,
            "Вакансия «Go-разработчик» снова опубликована"
        );
        assert_eq!(
            vacancy.body,
            "Модератор отменил снятие: «Ошибка модерации»."
        );
        assert_eq!(vacancy.link, Some(format!("/employer/vacancies/{id}")));
        let freeze = lifted(None, None);
        assert_eq!(freeze.title, "Приглашения снова доступны");
        assert_eq!(
            freeze.body,
            "Модератор снял приостановку досрочно: «Ошибка модерации». Квота — 20 приглашений за 7 дней."
        );
        assert_eq!(freeze.link.as_deref(), Some("/employer/trust"));
    }
}
