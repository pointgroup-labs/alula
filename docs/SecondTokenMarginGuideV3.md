# Second Token as Margin Guide — V3 (Single-Anchor Multiply)

> **Замінює `SecondTokenMarginGuideV2.md`**. V2 був майже правильний, але всередині `Borrow.amount` лишався *борг-fee gross-up* (`ceil(flash_repay × 10_000 / (10_000 − borrow_fee_bps))`) — фактично залишковий «фантомний» борг рівний значенню borrow_fee. V3 цього позбавляється: при стандартній multiply-friendly конфігурації пулу (`borrow_fee_bps == 0`, `add_collateral_fee_bps == 0`) флоу скорочується до **4 запитів з єдиною AddCollateral-якоркою**, і всі чотири літерали ідеально відповідають кінцевому стану обовʼязання.
>
> **Жодних змін у контракті не потрібно.** Зміна суто SDK-side.

---

## Передумови (must verify before submit)

Перед формуванням V3-батча SDK мусить переконатись, що обидва пули задовольняють умови нижче. Якщо хоч одна не виконана — **fall back до V2** (або відмова від multiply на цьому маркеті).

1. **`xlm_pool.config.fee_config.add_collateral_fee_bps == 0`** — інакше `process_add_collateral` зарахує в обовʼязання `amount − fee_sum` (`processors.rs:387-388`, `obligation.rs:635`, `pool.rs:710`). Інваріант "колатерал = `margin + Y`" перестане діяти.
2. **`usdc_pool.config.fee_config.borrow_fee_bps == 0`** — це **нова, ключова відмінність V3**. Лише при цій умові `process_borrow` видає в гаманець рівно `Borrow.amount` без слайсу (`obligation.rs:585`, `processors.rs:349-350`), тож `Borrow.amount` можна виставити рівним `flash_repay` без жодного gross-upʼу. Якщо пул бере borrow fee — `borrower_to_receive < flash_repay`, і FlashRepay реверне.
3. **Пули не міняли `borrow_fee_bps`/`flash_loan_fee_bps`/`add_collateral_fee_bps`** між квотою і submit — SDK читає актуальні значення з `pool.config.fee_config`.
4. **`referrer = None`** — захисна вимога. Технічно, при `borrow_fee_bps == 0` referrer-slice (`obligation.rs:1413, 1425`) сам виходить нульовим, бо це частка від `fee_sum == 0`. Але краще не залежати від цього: явно передавайте `referrer = None`.
5. **`pool.config.health_config.min_collateral_value_cents`** дотриманий фінальною позицією. Інакше `process_borrow` реверне попри проходження по LTV.
6. **`usdc_pool.total_available ≥ X`** на момент submit — інакше `process_flash_borrow` (`processors.rs:530`) реверне на `require_total_available`.
7. **На обовʼязанні немає активних bad-debt cover-реквестів** — `require_no_active_cover_bad_debt_requests_exists` (`processors.rs:332, 380`) реверне `Borrow` і `AddCollateral`, якщо такі є.
8. **`pool.config.status` має `BORROW_ENABLED | FLASH_LOAN_ENABLED | ADD_COLLATERAL_ENABLED`** на відповідних пулах — feature-flag перевірки в `pool.rs:504-520`.

Якщо хоч одна з умов не виконана — **зупинитись і не сабмітити**.

---

## Ключова ідея

V3 опирається на дві властивості, які `processors.rs` забезпечує безкоштовно:

1. **`process_flash_borrow` фізично переказує токен у гаманець *відразу***, всередині самого реквесту (`processors.rs:536`). Кошти з flash доступні гаманцю вже до наступного реквесту в батчі.
2. **`process_add_collateral` лише *черзить* user-transfer**, але **відразу персистить обовʼязання** з оновленим `collateral` (`processors.rs:390`). Перевірка балансу гаманця відбувається тільки на кроці `execute_transfers` в кінці батча. Тому ми можемо `AddCollateral(margin + Y)` *до* того, як токени фактично прийдуть з свопу — головне, щоб до **end-of-batch** вони у гаманці були.

Звідси випливає: **немає сенсу робити дві окремі AddCollateral**. Достатньо однієї, яка одразу анкорить **і** початковий margin, **і** слипедж-флур `Y` свопу — це і є той самий «маленький Deposit», який тримає всю детермінованість позиції.

---

## Схема операцій (V3)

```
1. FlashBorrow(USDC, X)                              // wallet після: margin XLM, X USDC
2. SwapExactTokens(USDC → XLM,
                   amount_in     = X,
                   min_amount_out = Y)                // wallet після: (margin + ≥Y) XLM, 0 USDC
3. AddCollateral(XLM, margin + Y)                    // queues pull (margin+Y) XLM,
                                                      // персистить obligation.collateral = margin+Y
4. Borrow(USDC, X + flash_fee)                       // queues market→wallet (X + flash_fee) USDC,
                                                      // персистить obligation.debt = X + flash_fee
   // далі автоматично виконається execute_transfers (request.rs:159):
   //   1) user→market: pulls (margin+Y) XLM з гаманця → wallet: (≥0 XLM bonus), 0 USDC
   //   2) market→user: sends (X+flash_fee) USDC у гаманець → wallet: bonus XLM, X+flash_fee USDC
   //   3) flash repay: pulls (X+flash_fee) USDC з гаманця → wallet: bonus XLM, 0 USDC ✓
```

Де:

- `margin` — кількість XLM, яку користувач кладе зі свого гаманця як початковий капітал.
- `Y = floor((L − 1) × margin × (1 − slippage_pct))` — найгірший прийнятний обʼєм XLM зі свопу, тобто слипедж-флур.
- `X = ceil(swap_provider.get_amount_in(path = [USDC, XLM], amount_out = Y) × (1 + safety_bps))` — стільки USDC, щоб DEX гарантовано видав ≥ Y XLM навіть під помірно несприятливою ціною. `safety_bps` (1–5 bps) страхує від раундингу всередині DEXʼу.
- `flash_fee = ceil(X × usdc_pool.fee_config.flash_loan_fee_bps / 10_000)` (`request.rs:170-172`).
- `borrow_amount = X + flash_fee` — **точно стільки**, скільки треба для flash repay. Жодного gross-upʼу, бо за передумовою `borrow_fee_bps == 0`.

**Порядок реквестів обовʼязковий і не може бути змінений.** Якщо поставити `AddCollateral` перед `FlashBorrow`, pre-FlashBorrow flush на `processors.rs:113` спробує фізично потягнути `(margin + Y)` XLM з гаманця, у якому є тільки `margin` → revert. Аналогічно `Borrow` не можна ставити перед `AddCollateral`, бо `check_health` побачить `collateral = margin` (тільки margin або 0) проти боргу `X + flash_fee` і не пройде LTV-перевірку.

---

## Інваріанти кінцевого стану

| Сценарій             | Реальний swap output | Гаманець після свопу | Гаманець після execute_transfers | Кінцевий стан позиції                |
| -------------------- | -------------------- | -------------------- | -------------------------------- | ------------------------------------ |
| Floor-on-money       | `Y` XLM              | `margin + Y` XLM     | `0` XLM                          | `margin + Y` колат, `X + flash_fee` борг |
| Favorable (`Y + Δ`)  | `Y + Δ` XLM          | `margin + Y + Δ` XLM | `Δ` XLM (бонус)                  | `margin + Y` колат, `X + flash_fee` борг — **той самий** |
| Adverse (`< Y`)      | swap reverts on `processors.rs:1036` | —                    | —                                | tx atomically rolled back            |

Ключові висновки:

1. **Колатерал точний** — `margin + Y` XLM в кожному успішному виконанні. Один-єдиний літерал в `AddCollateral.amount`.
2. **Борг точний** — `X + flash_fee` USDC в кожному успішному виконанні. Один-єдиний літерал в `Borrow.amount`. **Жодного фантомного боргу.**
3. **Позитивний слипедж = подарунок користувачу в XLM** (активі, на який він і так іде в long).
4. **`open_ltv` перевіряється проти точного фінального стану** через `obligation.borrow → compute_max_healthy_debt_added_amount / max_healthy_borrow_added_amount` (`obligation.rs:263-338`, `obligation.rs:559-575`) — крок 4 запускається після того, як крок 3 уже персистив `collateral = margin + Y` через `obligation.set` на `processors.rs:390`. Не пройшло — атомарне rollback.

---

## Як рахувати параметри (формули для SDK)

Вхідні дані від користувача:

- `margin` — у XLM (мінімальних одиницях, decimals=7)
- `L` — цільове плече (наприклад, `2.0` для 2x)
- `slippage_pct` — толерантність до слипеджу (наприклад, `0.005` для 0.5%)

Розрахунок:

```text
target_collateral_to_add = (L - 1) × margin
Y                        = floor(target_collateral_to_add × (1 - slippage_pct))   // min_amount_out
X_quote                  = swap_provider.get_amount_in(path = [USDC, XLM],
                                                      amount_out = Y)
X                        = ceil(X_quote × (1 + 0.0005))                            // +5 bps round-safety
flash_loan_fee_bps       = usdc_pool.config.fee_config.flash_loan_fee_bps
flash_fee                = ceil(X × flash_loan_fee_bps / 10_000)                   // request.rs:170-172
borrow_amount            = X + flash_fee                                           // ← exact, no gross-up
collateral_amount        = margin + Y                                              // ← single AddCollateral literal
```

**Перевірка перед сабмітом** (off-chain sanity, щоб не платити газ за приречений батч).

`process_borrow → obligation.borrow → compute_max_healthy_debt_added_amount` (`obligation.rs:263-338`, `obligation.rs:559-575`) використовує:

- зваження кожного колатеру через його `open_ltv_bps` (а не «глобальний» open_ltv);
- зваження боргу через `liability_factor_bps` пулу (`pool.rs:944`);
- віднімання `min_collateral_value_cents` як абсолютного floorʼу.

Грубий sanity-чек:

```text
weighted_collateral_value = (margin + Y) × xlm_oracle_price × xlm_pool.open_ltv_bps / 10_000
weighted_debt_value       = (X + flash_fee) × usdc_oracle_price × 10_000 / usdc_pool.liability_factor_bps

assert weighted_collateral_value ≥ weighted_debt_value
assert (margin + Y) × xlm_oracle_price ≥ pool.config.health_config.min_collateral_value_cents
```

Це наближення, а не точне відтворення `check_health` — для точного pre-check виконуйте локальну симуляцію формул з `obligation.rs:263-293`. Або просто покладайтесь на on-chain-revert як на atomic safety net.

---

## Приклад робочої команди

Припустимо:

- `margin` = 100 XLM = `1_000_000_000` (7 decimals)
- `L` = 2x
- `slippage_pct` = 0.5%
- DEX-quote: `swap_provider.get_amount_in([USDC, XLM], 995_000_000)` → `220_505_319` USDC
- `flash_loan_fee_bps` = 9 (з `usdc_pool.config.fee_config.flash_loan_fee_bps`)
- `borrow_fee_bps` = 0 (передумова — інакше не починаємо)
- `add_collateral_fee_bps` = 0 (передумова — інакше не починаємо)

Тоді:

```text
target_collateral_to_add = 1_000_000_000
Y                        = floor(1_000_000_000 × 0.995)   = 995_000_000
X_quote                  = 220_505_319
X                        = ceil(220_505_319 × 1.0005)     = 220_615_572   ← flash principal & swap input
flash_fee                = ceil(220_615_572 × 9 / 10_000) = 198_555
borrow_amount            = X + flash_fee                  = 220_814_127   ← rівно стільки треба для FlashRepay
collateral_amount        = margin + Y                     = 1_995_000_000 ← єдина AddCollateral-якорь
```

```bash
scitm2 --id bot_main -- submit_requests_batch --requests '[
    {
      "FlashBorrow": {
        "amount": "220615572",
        "pool_address": "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
      }
    },
    {
      "SwapExactTokens": {
        "amount_in":      "220615572",
        "min_amount_out": "995000000",
        "path": [
          "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
          "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        ],
        "swap_provider": "CBCCMQFUDEEEMWAXUDB5ULBJA2XR4CWHKXMBOWJPBPOLPLVJDSP5QIUB"
      }
    },
    {
      "AddCollateral": {
        "amount": "1995000000",
        "pool_address": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
      }
    },
    {
      "Borrow": {
        "amount": "220814127",
        "pool_address": "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
      }
    }
]' --user "$m2"
```

**Інваріанти, які SDK мусить тримати** (зашити як assert-и в білдері батча):

- `FlashBorrow.amount = X = SwapExactTokens.amount_in` — три літерали з різних реквестів **повинні бути ідентичні**. Якщо ні — flash_repay буде розрахований з іншого принципала і реверне.
- `AddCollateral.amount = margin + SwapExactTokens.min_amount_out` — це і є якір детермінізму. Якщо `AddCollateral.amount > margin + min_amount_out` — швидко за best-case rate буде ОК, але за floor-rate end-of-batch fallne на нестачу XLM в гаманці. Якщо `AddCollateral.amount < margin + min_amount_out` — частина свопного XLM лишиться у гаманці бонусом понад очікуваний (формально не помилка, але порушує семантику «весь Y → колатерал»).
- `Borrow.amount = FlashBorrow.amount + ceil(FlashBorrow.amount × flash_loan_fee_bps / 10_000)` — точно. Жодних додаткових gross-upʼів.

---

## Чому V3 краще за V2

| Аспект                                       | V2                                                                                       | V3                                                              |
| -------------------------------------------- | ---------------------------------------------------------------------------------------- | --------------------------------------------------------------- |
| Кількість запитів                            | 5                                                                                        | **4**                                                           |
| Кількість `AddCollateral`                    | 2 (`margin`, потім `Y`)                                                                  | **1** (`margin + Y`)                                            |
| `Borrow.amount`                              | `ceil(flash_repay × 10_000 / (10_000 − borrow_fee_bps))` — gross-up                    | **`X + flash_fee` — exact**                                     |
| Фантомний борг від borrow_fee                | так (на borrow_fee × flash_repay / 10_000 USDC)                                          | **нуль**                                                        |
| Якір детермінованості                        | другий `AddCollateral.amount = min_amount_out`                                           | **єдиний `AddCollateral.amount = margin + min_amount_out`**     |
| Передумови                                   | `add_collateral_fee_bps == 0`                                                            | `add_collateral_fee_bps == 0` **+ `borrow_fee_bps == 0`**       |
| Працює на маркетах з ненульовим borrow_fee   | так                                                                                      | **ні** — fall back до V2                                        |
| Складність білдера батча                     | вища (gross-up, дві AddCollateral)                                                       | нижча                                                           |

V3 — це V2, спрощений до ідеалу для multiply-friendly маркетів. Якщо маркет конфігурується з нульовими fee на borrow і add_collateral (найбільш типовий випадок для активної multiply-стратегії), **V3 треба використовувати замість V2**. Якщо ж borrow_fee > 0 — V2 лишається коректним fallbackʼом.

---

## Чому V1 лишається неправильним

(Залишено для повноти; деталі — в `SecondTokenMarginGuideV2.md`.)

V1 будував `Borrow.amount` через worst-case `SwapForExactTokens.max_amount_in` → буфер слипеджу матеріалізовувався як **надлишковий USDC у гаманці** і **інфляція боргу в обовʼязанні**. V2 і V3 фіксують це структурно, прив'язуючи якір детермінованості до `min_amount_out` свопу замість `max_amount_in`.

---

## Інваріанти, які варто перевіряти end-to-end тестом

Покрито 23-test integration suite в **`tests/src/multiply_v3.rs`**. Запуск:

```bash
cargo nextest run multiply_v3 --workspace --lib
```

| # | Тест | Що перевіряє |
| - | ---- | ------------ |
| 1  | `v3_floor_on_money_produces_exact_position_no_bonus` | DEX повертає рівно `Y`; колат і борг — точні літерали |
| 2  | `v3_favorable_slippage_yields_xlm_bonus_position_unchanged` | +5% slippage → бонус у гаманці, позиція не змінилась |
| 3  | `v3_adverse_slippage_reverts_atomically_no_state_change` | −1% adverse → atomic rollback |
| 4  | `v3_wrong_order_addcollateral_before_flashborrow_reverts` | Порядок реквестів обовʼязковий |
| 5  | `v3_two_users_get_identical_positions_under_different_slippage` | Детермінованість позицій між юзерами |
| 6  | `v3_extreme_favorable_slippage_2x_position_still_exact` | +100% slippage — позиція все ще точна |
| 7  | `v3_just_below_floor_one_unit_short_reverts` | Boundary: 24 µ нижче `Y` → revert |
| 8  | `v3_large_slippage_with_low_floor_multiply_succeeds` | −50% slippage, але `Y` сконфігуровано низько → succeeds |
| 9  | `v3_skewed_quote_gold_expensive_position_exact_at_floor` | Котирування `X = 2Y` (gold expensive) |
| 10 | `v3_skewed_quote_gold_cheap_with_favorable_slippage` | Котирування `Y = 4X` (gold cheap) + favorable |
| 11 | `v3_silently_milks_user_when_borrow_fee_bps_nonzero` | **Critical**: тихий витік fee при `borrow_fee_bps != 0` |
| 12 | `v3_breaks_determinism_when_add_collateral_fee_bps_nonzero` | Колатерал недодається при ненульовому fee |
| 13 | `v3_under_open_ltv_succeeds` | LTV ~60% (під дефолтним open_ltv=70%) |
| 14 | `v3_over_open_ltv_reverts_with_unhealthy_operation` | LTV >70% → `MCError::UnhealthyOperation` |
| 15 | `v3_with_unregistered_referrer_still_deterministic` | Referrer ≠ None, але не зареєстрований у пулі → інваріант тримається |
| 16 | `v3_multi_hop_swap_path_works` | Path `[USDC, BTC, GOLD]` (3-hop) |
| 17 | `v3_reverts_when_flash_loan_disabled` | Feature-flag enforcement |
| 18 | `v3_reverts_when_pool_lacks_flash_liquidity` | Pool reserve exhaustion |
| 19 | `v3_with_custom_flash_loan_fee_bps_works` | `flash_loan_fee_bps = 9` (V1-era налаштування) |
| 20 | `v3_with_zero_flash_loan_fee_works` | `flash_loan_fee_bps = 0` edge |
| 21 | `v3_adds_to_preexisting_collateral_position` | V3 поверх існуючого колатеру |
| 22 | `v3_adds_to_preexisting_borrow_position` | V3 поверх існуючого боргу |
| 23 | `v3_with_tiny_amounts_handles_rounding_correctly` | margin = X = Y = 1 µ |

Wallet net-deltas, які реально тримаються (важливо: не плутати з зануленням):

- **USDC wallet**: net delta = `0` (Borrow → FlashRepay netting), за умови `borrow_fee_bps == 0`. При ненульовому borrow fee — wallet тихо втрачає `borrow_fee` (тест 11).
- **XLM wallet floor-on-money**: net delta = `−margin`. Swap mint `Y` XLM повністю compensує AddCollateral pull `(margin + Y)`, лишається `−margin`.
- **XLM wallet favorable**: net delta = `−margin + (actual_swap_output − Y)`. Бонус = surplus.
- **XLM wallet adverse**: net delta = `0` (atomic revert).

---

## Чого V3 свідомо НЕ робить

1. **Автоматичний депозит позитивного слипеджу.** Бонусний XLM лишається в гаманці. Щоб його авто-задепонувати, потрібен новий request-варіант `AddCollateralAll` (контрактна зміна).
2. **End-of-batch health-factor assert.** Без `Request::AssertObligationHealth { min_hf_bps }` V3 покладається на `open_ltv`-перевірку в `process_borrow` — але цього достатньо, бо фінальний стан детермінований.
3. **Підтримка маркетів з ненульовим `borrow_fee_bps` без fallback.** Якщо команда хоче multiply на borrow-fee-bearing маркеті — використовуйте V2.
4. **Single-call ентрі поінт `deposit_with_leverage`.** Залишається можливим майбутнім кроком (як периферійний контракт).

---

## TL;DR для рев'ю

- **Зміна суто SDK-side**, контракт не чіпаємо.
- **Передумови**: `borrow_fee_bps == 0` AND `add_collateral_fee_bps == 0` AND `referrer = None`.
- **4 запити** в порядку: `FlashBorrow → SwapExactTokens → AddCollateral → Borrow`. Порядок обовʼязковий.
- **Інваріант 1**: `FlashBorrow.amount = SwapExactTokens.amount_in = X`.
- **Інваріант 2**: `AddCollateral.amount = margin + SwapExactTokens.min_amount_out`.
- **Інваріант 3**: `Borrow.amount = X + ceil(X × flash_loan_fee_bps / 10_000)` — точно, без gross-upʼу.
- **Слипедж матеріалізується як XLM-бонус у гаманці**, а не як надлишковий борг чи USDC-дріб'язок.
- **Перед мерджем** — прогнати п'ятикейсовий тест із розділу «Інваріанти».
- **Якщо `borrow_fee_bps > 0`** — fall back до V2.
