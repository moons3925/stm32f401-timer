#![no_std]
#![no_main]

use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
use cortex_m_rt::entry;
use stm32f4::stm32f401;
use stm32f4::stm32f401::interrupt;  // (1)割り込みを使う

#[entry]
fn main() -> ! {
    unsafe {
        stm32f401::NVIC::unmask(stm32f401::interrupt::TIM2);    // (2)TIM2 NVIC割り込み許可
    }
    let dp = stm32f401::Peripherals::take().unwrap();   // (3)Peripheralsの取得

    // PLLSRC = HSI (default)
    dp.RCC.pllcfgr.modify(|_, w| w.pllp().div4());  // (4)P=4
    dp.RCC.pllcfgr.modify(|_, w| unsafe { w.plln().bits(336) });    // (5)N=336
    // PLLM = 16 (default)

    dp.RCC.cfgr.modify(|_, w| w.ppre1().div2());    // (6) APB1 PSC = 1/2

    dp.RCC.cr.modify(|_, w| w.pllon().on());    // (7)PLL On
    while dp.RCC.cr.read().pllrdy().is_not_ready() {    // (8)安定するまで待つ
        // PLLがロックするまで待つ (PLLRDY)
    }

    // データシートのテーブル15より
    dp.FLASH.acr.modify(|_,w| w.latency().bits(2));    // (9)レイテンシの設定: 2ウェイト (3.3V, 84MHz)

    dp.RCC.cfgr.modify(|_,w| w.sw().pll()); // (10)sysclk = PLL
    while !dp.RCC.cfgr.read().sws().is_pll() {  // (11)SWS システムクロックソースがPLLになるまで待つ
    }

    dp.RCC.apb1enr.modify(|_,w| w.tim2en().enabled());  // (12)TIM2のクロックを有効にする

    let tim2 = &dp.TIM2;    // (13)TIM2の取得
    tim2.psc.modify(|_, w| unsafe { w.bits(84 - 1) });   // (14)プリスケーラの設定
    tim2.arr.modify(|_, w| unsafe { w.bits(1000000 - 1) }); // (15)ロードするカウント値
    tim2.dier.modify(|_, w| w.uie().enabled()); // (16)更新割り込み有効
    tim2.cr1.modify(|_, w| w.cen().enabled());  // (17)カウンタ有効

    dp.RCC.ahb1enr.modify(|_, w| w.gpioaen().enabled());    // (18)GPIOAのクロックを有効にする
    let gpioa = &dp.GPIOA;                                  // (19)GPIOAの取得
    gpioa.moder.modify(|_, w| w.moder5().output());         // (20)GPIOA5を出力に設定    
    loop {
        // your code goes here
    }
}

#[interrupt]    // (21)割り込みの指定
fn TIM2() {     // (22)TIM2割り込みハンドラ
    unsafe {
        let dp = stm32f401::Peripherals::steal();   // (23)Peripheralsの取得
        dp.TIM2.sr.modify(|_, w| w.uif().clear());  // (24)更新フラグのクリア
        dp.GPIOA.odr.modify(|r, w| w.odr5().bit(r.odr5().is_low()));    // (25)GPIOAの出力を反転する
    }
}
