# Second Token as Margin Guide (Cross-Asset Looping)

Наша реалізація лупінгу з другим токеном як маржею працює через `submit_requests_batch` ендпойнт. Цей процес дозволяє користувачу використовувати один токен (наприклад, USDC) як початковий капітал, але створити левереджовану позицію в іншому токені (наприклад, XLM).

## Концепція

Замість "borrow as margin" (де ти позичаєш той самий токен, що депозитиш), тут користувач:
1. Має початковий токен А (USDC)
2. Хоче створити левереджовану позицію в токені A (USDC)
3. Flash borrow токен A (XLM) кількістю (L-1), де L - плече
4. Депозитить initial amount + flash borrowed в маркет
5. Позичає в маркета стільки токена Б(USDC), що якщо його свопнути на токен A, то в тебе буде достатньо токену щоб зробити FlashRepay(flash borrow amount + flash borrow fees)
6. FlashRepay(робиться автоматично)

---

## Приклад робочої команди

Припустимо, користувач має:
- **Початковий капітал**: 100 XLM
- **Leverage multiplier**: 2x
- **Результат**: закручена позиція з 200 XLM в депозиті

```bash
scitm2 --id bot_main -- submit_requests_batch --requests '[
    {
      "FlashBorrow": {
        "amount": "1000000000", // borrow 100
        "pool_address": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
      }
    },
    {
      "Deposit": {
        "amount": "2000000000", // 2x закинули
        "pool_address": "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
      }
    },
    {
      "Borrow": {
        "amount": "250368069",
        "pool_address": "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA"
      }
    },
    {
      "SwapForExactTokens": {
        "max_amount_in": "250368069",
        "amount_out": "1000599800",
        "path": [
          "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
          "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
        ],
        "swap_provider": "CBCCMQFUDEEEMWAXUDB5ULBJA2XR4CWHKXMBOWJPBPOLPLVJDSP5QIUB"
      }
    }
]' --user "$m2"
```

## Як рахувати параметри

### 1. Initial Amount (в токені А)
Це кількість токенів А (USDC), які користувач готовий використати як початковий капітал.

### 2. Leverage Multiplier(так само як в попередньому кейсі)
```
Max leverage multiplier = 1/(1 - openLTV) * 0.8
```

### 3. Borrow amount

Тобі треба на swap_provider викликати `get_amount_in`(для іншого виду закрутки ми використовувати `get_amount_out`)

250117951

250368069

На нього в amount_out тобі треба передати
((1000000000 * 1.0009) / (1 - 0.0005)) * (1 + slippage_percent)

1.0009 - бо в нас 9bps - flash loan fee
0.0005 - бо в нас 5bps - borrow fee
ну і slippage має бути в залежності від налаштувань юзера, як завжди

в мене вийшло 1051470735 при нульовому сліпеджі

а сам swap_provider мені повернув ["220505319","1051470735"], де перше значення - те, яке тобі треба взяти


