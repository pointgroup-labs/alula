# Second Token as Margin Guide — V2 (Slippage-Exact Multiply)

> **Замінює `SecondTokenMarginGuide.md`**. Стара схема (flash-borrow колатерального активу + `SwapForExactTokens` + великий буфер у `Borrow.amount`) залишала користувача з **завищеним боргом** — буфер слипеджу матеріалізувався як надлишковий USDC у позиції. Нова схема (flash-borrow боргового активу + `SwapExactTokens` + `AddCollateral.amount = min_amount_out`) дає **детермінований кінцевий стан позиції** — позитивний слипедж переходить у гаманець користувача як XLM-бонус, а борг та колатерал у обʼязанні точні.
>
> **Жодних змін у контракті не потрібно.** Вся різниця — у тому, як SDK заповнює літерали в `Request`-ах.

---

## Передумови (must verify before submit)

Перш ніж будувати V2-батч, SDK мусить переконатись, що обидва пули задовольняють такі умови. Якщо ні — формули нижче не діють і батч може ревертнути або дати неочікувану позицію.

1. **`xlm_pool.config.fee_config.add_collateral_fee_bps == 0`** — інакше `process_add_collateral` зарахує в обовʼязання `amount − fee_sum`, а не `amount` (див. `obligation.rs:635`, `pool.rs:710`). Інваріант "колатерал = `margin + Y`" перестане діяти; формули потребують переписування.
2. **Пули не міняли `borrow_fee_bps`/`flash_loan_fee_bps`** між моментом квоти і submit — SDK читає актуальні значення з `pool.config.fee_config`.
3. **`referrer = None`** для всіх викликів `submit_requests_batch` у V2. Реферальна частка вираховується з `Borrow.amount` як slice того самого `fee_sum` (`obligation.rs:1425`), і якщо її не врахувати — `borrower_to_receive` буде менший за `flash_repay`, FlashRepay реверне.
4. **`pool.config.health_config.min_collateral_value_cents`** дотриманий фінальною позицією. Інакше `process_borrow` реверне попри проходження по LTV.

Якщо хоч одна з умов не виконана — зупинитись і не сабмітити.

---

## Ключова ідея

`min_amount_out` свопу — це **єдине джерело правди** для слипеджу в усій транзакції. Кожен літерал, який стосується пост-свопного значення (передусім `AddCollateral.amount`), має дорівнювати **саме `min_amount_out`**, а не «очікуваному виходу» зі свопу.

Якщо реальний свап повертає більше — надлишок залишається в гаманці користувача як подарунок (positive slippage). Якщо повертає менше — свап атомарно ревертить через `min_amount_out`-перевірку всередині `process_swap_exact` (`processors.rs:1036`). У будь-якому успішному виконанні позиція в обовʼязанні **бітово детермінована**.

---

## Схема операцій

```
1. AddCollateral(XLM, margin)                         // exact margin from wallet
2. FlashBorrow(USDC, X)                               // flash debt asset
3. SwapExactTokens(USDC → XLM,
                   amount_in     = X,
                   min_amount_out = Y)                 // Y = floor user accepts
4. AddCollateral(XLM, Y)                              // ← KEY: literally Y, same number as min_amount_out
5. Borrow(USDC, X + flash_fee)                        // exact, just enough to repay flash
   (FlashRepay auto-fires inside execute_transfers)
```

Де:

- `margin` — кількість XLM, яку користувач кладе зі свого гаманця як початковий капітал.
- `Y = (L − 1) × margin × (1 − slippage_pct)` — найгірший прийнятний обʼєм XLM, який має повернути свап. Це і є слипедж-флур.
- `X = swap_provider.get_amount_in(amount_out = Y) × (1 + small_safety_bps)` — рівно стільки USDC, скільки потрібно DEXʼу, щоб гарантовано видати ≥ Y XLM навіть під помірно несприятливою ціною. `small_safety_bps` (1–5 bps) тільки страхує від раундингу всередині DEXʼу.
- `flash_fee = X × pool.fee_config.flash_loan_fee_bps / BPS_FACTOR` (округлення вгору, як у `request.rs:170-172`).

---

## Чому позиція точна — покейсовий аналіз

Після кроку 3 у гаманці користувача гарантовано **щонайменше `Y` XLM** (інакше свап би ревертнув). Три сценарії:

| Реальний вихід свопу | Гаманець після кроку 3 | Крок 4 забирає | Гаманець після кроку 4 | Кінцевий стан позиції                 |
| -------------------- | ---------------------- | -------------- | ---------------------- | ------------------------------------- |
| Рівно `Y`            | `Y` XLM                | `Y` XLM        | 0 XLM                  | `margin + Y` колат, `X + fee` борг    |
| `Y + Δ` (favorable)  | `Y + Δ` XLM            | `Y` XLM        | `Δ` XLM (бонус)        | `margin + Y` колат, `X + fee` борг — **той самий** |
| `< Y`                | (свап ревертнув)       | —              | —                      | tx atomically rolled back             |

**Властивості, які звідси випливають:**

1. **Борг точний** — `X + flash_fee` USDC у кожному успішному виконанні. Жодного над-займу, жодних «зайвих» bps на borrow fee.
2. **Колатерал точний** — `margin + Y` XLM у кожному успішному виконанні. Жодного тихого «підсмоктування» з гаманця.
3. **Позитивний слипедж = подарунок користувачу** в активі, на який він і так іде в long (XLM). Контракт нічого зайвого не забирає.
4. **`open_ltv` перевіряється проти точного фінального стану** — `(X + flash_fee)` боргу проти `(margin + Y)` колатеру. Якщо позиція не проходить `open_ltv`, ревертить крок 5 → атомарне відкочування.

---

## Як рахувати параметри (формули для SDK)

Вхідні дані від користувача:

- `margin` — у XLM (мінімальних одиницях, decimals=7)
- `L` — цільове плече (наприклад, `2.0` для 2x)
- `slippage_pct` — толерантність користувача до слипеджу (наприклад, `0.005` для 0.5%)

Розрахунок:

```text
target_collateral_to_add = (L - 1) × margin
Y                        = floor(target_collateral_to_add × (1 - slippage_pct))    // min_amount_out
X_quote                  = swap_provider.get_amount_in(path = [USDC, XLM],
                                                      amount_out = Y)
X                        = ceil(X_quote × (1 + 0.0005))                            // +5 bps round-safety
flash_fee                = ceil(X × flash_loan_fee_bps / 10_000)                   // request.rs:170-172
flash_repay              = X + flash_fee                                           // принципал, який треба
                                                                                    // повернути в FlashRepay
borrow_fee_bps           = usdc_pool.config.fee_config.borrow_fee_bps
borrow_amount            = ceil(flash_repay × 10_000 / (10_000 - borrow_fee_bps))  // gross-up:
                                                                                    // process_borrow видасть
                                                                                    // borrower_to_receive
                                                                                    // = borrow_amount - fee
                                                                                    // має дорівнювати flash_repay
```

**Чому gross-up.** `process_borrow` додає в борг повне `Borrow.amount`, але переводить у гаманець `Borrow.amount − ceil(Borrow.amount × borrow_fee_bps / 10_000)` (див. `obligation.rs:585`, `processors.rs:349`). Якщо `borrow_fee_bps == 0`, то `borrow_amount == flash_repay`. Якщо ж пул бере borrow fee, то без gross-upʼу гаманець отримає менше, ніж потрібно для FlashRepay, і батч ревертне.

**Перевірка перед сабмітом** (off-chain sanity, щоб не платити газ за приречений батч).

Спрощена «raw LTV»-формула, яку я наводив у попередній редакції цього гайда, **не відповідає тому, що насправді перевіряє контракт**. `process_borrow → obligation.borrow → check_health` (`obligation.rs:415`, `obligation.rs:263`, `obligation.rs:293`) використовує:

- зваження кожного колатеру через його `open_ltv_bps` (а не «глобальний» open_ltv);
- зваження боргу через `liability_factor_bps` пулу (`pool.rs:944`);
- віднімання `min_collateral_value_cents` як абсолютного floorʼу.

Тому коректний off-chain pre-check — це **локальна симуляція тих самих формул** з `obligation.rs:263-293`, або просто покладання на on-chain ревертну перевірку як на atomic safety net (батч просто реверне, ефекту нема). Якщо все ж хочете грубий sanity-чек:

```text
weighted_collateral_value = (margin + Y) × xlm_oracle_price × xlm_pool.open_ltv_bps / 10_000
weighted_debt_value       = borrow_amount × usdc_oracle_price × 10_000 / usdc_pool.liability_factor_bps

assert weighted_collateral_value ≥ weighted_debt_value
assert (margin + Y) × xlm_oracle_price ≥ pool.config.health_config.min_collateral_value_cents
```

Це наближення, а не точне відтворення `check_health` — звертайтесь до `obligation.rs` за точною формулою.

Максимальне досяжне плече `L_max` від `slippage_pct` залежить нелінійно через ту саму систему (зважені колатерал/борг + min-collateral floor). Точно його рахувати — фітом до тих самих формул; для UI достатньо ітеративного підбору `L` з кроком 0.1× з пере-запуском sanity-чека.

---

## Приклад робочої команди

Припустимо:

- `margin` = 100 XLM = `1_000_000_000` (7 decimals)
- `L` = 2x
- `slippage_pct` = 0.5%
- DEX-quote: `swap_provider.get_amount_in([USDC, XLM], 995_000_000)` → `220_505_319` USDC
- `flash_loan_fee_bps` = 9 (з `usdc_pool.config.fee_config.flash_loan_fee_bps`)
- `borrow_fee_bps` = 5 (з `usdc_pool.config.fee_config.borrow_fee_bps`)
- `add_collateral_fee_bps` = 0 (передумова з розділу вище — інакше не починаємо)

Тоді:

```text
target_collateral_to_add = 1_000_000_000
Y                        = floor(1_000_000_000 × 0.995)        = 995_000_000
X_quote                  = 220_505_319
X                        = ceil(220_505_319 × 1.0005)          = 220_615_572   ← flash principal
flash_fee                = ceil(220_615_572 × 9 / 10_000)      = 198_555
flash_repay              = X + flash_fee                        = 220_814_127   ← треба для FlashRepay
borrow_amount            = ceil(220_814_127 × 10_000 / 9_995)  = 220_924_590   ← gross-up для borrow fee
```

Перевірка gross-upʼу: `220_924_590 − ceil(220_924_590 × 5 / 10_000) = 220_924_590 − 110_463 = 220_814_127 = flash_repay ✓`.

```bash
scitm2 --id bot_main -- submit_requests_batch --requests '[
    {
      "AddCollateral": {
        "amount": "1000000000",
        "pool_address": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
      }
    },
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
        "amount": "995000000",
        "pool_address": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
      }
    },
    {
      "Borrow": {
        "amount": "220924590",
        "pool_address": "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
      }
    }
]' --user "$m2"
```

Зверніть увагу (інваріанти, які SDK мусить тримати):

- `FlashBorrow.amount = X = SwapExactTokens.amount_in = 220_615_572` — **усі три літерали однакові**. Якщо `FlashBorrow.amount` ≠ `X`, то FlashRepay буде розрахований з іншого принципала, і батч ревертне. Це найчастіша помилка — попередня редакція цього гайда мала саме її.
- `AddCollateral[1].amount (995_000_000) = SwapExactTokens.min_amount_out (995_000_000)` — ця ідентичність робить позицію детермінованою.
- `Borrow.amount (220_924_590) ≠ FlashBorrow.amount (220_615_572)` — borrow завжди трохи більший, бо включає `flash_fee` плюс gross-up на `borrow_fee_bps`. У випадку `borrow_fee_bps = 0` він дорівнює `flash_repay = 220_814_127`.

---

## Чому стара схема (V1) була неправильна

V1-флоу з `SecondTokenMarginGuide.md`:

```
FlashBorrow(XLM, 1_000_000_000)
Deposit    (XLM, 2_000_000_000)              // exact
Borrow     (USDC, 250_368_069)               // ← buffer baked here
SwapForExactTokens(USDC→XLM,
                   max_in = 250_368_069,
                   out    = 1_000_599_800)
```

Що ставалось при сприятливому слипеджі (DEX витрачав, скажімо, 220M USDC замість дозволених 250M):

- 30M USDC залишались у гаманці користувача як решта.
- В обовʼязанні ж було записано **повних 250M USDC боргу** — бо саме стільки видав крок `Borrow`.
- Користувач мусив вручну зробити окремий `Repay`, щоб привести позицію до інтенту. Без цього — переплачував **borrow fee на повну буферну суму** і мав вищий LTV ніж планував.

V2-флоу не має цього недоліку, бо борг сайзиться **точно під flash repay**, а не під «гірший випадок DEX-конвертації».

---

## Інваріанти, які варто перевіряти end-to-end тестом

Перш ніж викочувати V2 у прод, додайте до `tests/src/` тест, який покриває обидва сценарії слипеджу. Псевдокод:

```rust
// 1. Поганий випадок — точно мінімальний swap output
fixture.set_amm_rate(USDC -> XLM, exactly_at_floor);
fixture.submit_requests_batch(user, v2_batch);

let obl = fixture.get_obligation(user);
assert_eq!(obl.collateral(XLM), margin + Y);            // exact
assert_eq!(obl.debt(USDC),       X + flash_fee);        // exact
assert_eq!(fixture.balance(user, XLM), 0);              // no surplus
assert_eq!(fixture.balance(user, USDC), 0);             // no surplus

// 2. Сприятливий випадок — DEX дав на 5% більше
fixture.set_amm_rate(USDC -> XLM, floor * 1.05);
fixture.submit_requests_batch(user2, v2_batch);

let obl2 = fixture.get_obligation(user2);
assert_eq!(obl2.collateral(XLM), margin + Y);           // STILL exact
assert_eq!(obl2.debt(USDC),       X + flash_fee);       // STILL exact
assert!(fixture.balance(user2, XLM) > 0);               // bonus in wallet
assert_eq!(fixture.balance(user2, USDC), 0);

// 3. Несприятливий випадок — DEX дав менше за floor
fixture.set_amm_rate(USDC -> XLM, floor * 0.99);
let result = fixture.try_submit_requests_batch(user3, v2_batch);
assert!(result.is_err());                               // atomic revert
assert_eq!(fixture.get_obligation(user3), None);        // no state change
```

Якщо всі три сценарії проходять — схема працює саме так, як описано вище. Якщо хоч один валиться — **не випускайте в прод**, читайте трасу й розбирайтесь.

---

## Чого V2 свідомо НЕ робить

Щоб не плодити складність, у V2 свідомо не закладено таких речей (можуть бути пізніше):

1. **Автоматичний депозит позитивного слипеджу.** Бонусний XLM лишається в гаманці користувача. Якщо хочете автоматично закидати його в позицію — потрібен новий request-варіант `AddCollateralAll` з семантикою `i128::MAX` («забрати весь баланс цього токена з гаманця»). Це зміна контракту — окрема задача.
2. **End-of-batch health-factor assert.** `Request::AssertObligationHealth { min_hf_bps }` як defense-in-depth. Без нього V2 покладається на те, що `open_ltv`-перевірка в `process_borrow` достатня — а вона достатня, бо в V2 фінальний стан обовʼязання детермінований.
3. **Single-call ентрі поінт `deposit_with_leverage`.** Залишається можливим майбутнім кроком (як периферійний контракт), але V2 уже знімає основний біль і без нього.

---

## TL;DR для рев'ю

- **Зміна суто SDK-side**, контракт не чіпаємо.
- **Інваріант**: `AddCollateral.amount` (другий) ≡ `SwapExactTokens.min_amount_out`. Завжди.
- **Інваріант**: `Borrow.amount` = `FlashBorrow.amount + flash_fee`. Точно.
- **Слипедж матеріалізується як XLM-бонус у гаманці**, а не як надлишковий борг.
- **Перед мерджем** — прогнати трикейсовий тест із розділу «Інваріанти».
