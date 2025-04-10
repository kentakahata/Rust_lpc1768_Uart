#![no_std]
#![no_main]

use lpc1769;
use panic_halt as _; // you can put a breakpoint on `rust_begin_unwind` to catch panics
/*
    para compilar para hexa utilizar o "cargo build --release" e "cargo objcopy --release -- -O ihex firmware.hex"

*/

use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::entry;
use core::ptr::write_volatile;


fn timer_set(syst: &mut SYST, valor: u32) {
    for _ in 0..valor {
        while !syst.has_wrapped() {}
    }
}



#[entry]
fn main() -> ! {
    
    let p = lpc1769::Peripherals::take().unwrap();
    let crtxm = cortex_m::Peripherals::take().unwrap();
    
    
    p.SYSCON.scs.modify(|_, w| w.oscen().enabled());
    while p.SYSCON.scs.read().oscstat().bit_is_clear() {}

    // 2. Seleciona o Main Oscillator como fonte de clock
    p.SYSCON.clksrcsel.write(|w| w.clksrc().selects_the_main_osc());

    p.SYSCON.cclkcfg.modify(|_, w| {
        unsafe { w.cclksel().bits(3) }
    });


    p.SYSCON.pll0cfg.modify(|_, w| {
       unsafe { w.msel0().bits(99);
                w.nsel0().bits(5)
       }
    });

    p.SYSCON.pll0feed.write(|w| unsafe { w.bits(0xAA) });
    p.SYSCON.pll0feed.write(|w| unsafe { w.bits(0x55) });

    p.SYSCON.pll0con.write(|w| {
        w.plle0().set_bit()
    });

    p.SYSCON.pll0feed.write(|w| unsafe { w.bits(0xAA) });
    p.SYSCON.pll0feed.write(|w| unsafe { w.bits(0x55) });

    while p.SYSCON.pll0stat.read().plle0_stat().bit_is_clear() {}

    p.SYSCON.pll0con.write(|w| {
        w.plle0().set_bit();
        w.pllc0().set_bit()
    });
    
    p.SYSCON.pll0feed.write(|w| unsafe { w.bits(0xAA) });
    p.SYSCON.pll0feed.write(|w| unsafe { w.bits(0x55) });
    while p.SYSCON.pll0stat.read().plle0_stat().bit_is_clear()  || p.SYSCON.pll0stat.read().pllc0_stat().bit_is_clear(){}


    const GPIO3_OUT: *mut u32 = 0x2009_C063 as *mut u32;
    const GPIO3_PIN25_ADRS: u32 = 1<<1;

    unsafe{
        write_volatile(GPIO3_OUT, GPIO3_PIN25_ADRS);
    }
    
    let mut syst = crtxm.SYST;

    // configure the system timer to wrap around every second
    syst.set_clock_source(SystClkSource::Core);
    syst.set_reload(1_000_000); // 1s
    syst.enable_counter();


    p.SYSCON.pclksel0.modify(|_, w| {
        w.pclk_uart0().cclk_div_4()
    });

    p.PINCONNECT.pinsel0.modify(|_,w|{
        w.p0_2().txd0();
        w.p0_3().rxd0()
    });

    p.SYSCON.pconp.modify(|_, w| {
        w.pcuart0().set_bit()
    });

    p.UART0.lcr.write(|w| w.dlab().enable_access_to_div());

    p.UART0.dll().write(|w| unsafe { w.bits(163) }); // Divisor LSB
    p.UART0.dlm().write(|w| unsafe { w.bits(0) }); // Divisor MSB
    p.UART0.fdr.write(|w| unsafe {
        w.mulval().bits(1); 
        w.divaddval().bits(0)
    });
    

    p.UART0.lcr.write(|w| {
        w.wls()._8_bit_character_leng();
        w.sbs()._1_stop_bit_();
        w.pe().disable_parity_gener();
        w.dlab().disable_access_to_di()
    });
   
    p.UART0.fcr().write(|w| {
        w.fifoen().active_high_enable_f();
        w.rxfifores().writing_a_logic_1_to();
        w.txfifores().writing_a_logic_1_to()
    });
    
    let mut count = 0;
    
    p.GPIO.dir3.write(|w| w.pindir25().set_bit());

    loop {
        p.GPIO.set3.write(|w| w.pinset25().set_bit());
        timer_set(&mut syst, 50);
        p.GPIO.clr3.write(|w| w.pinclr25().set_bit());
        timer_set(&mut syst, 50);
        count += 1;
        if count == 10 {

            let message = b"hello world\n\r";
            for &byte in message {
                while p.UART0.lsr.read().thre().bit_is_clear() {}
    
                p.UART0.thr().write(|w| unsafe { w.bits(byte as u32) });
                
            }
            count = 0;
            
        }
    }

}
