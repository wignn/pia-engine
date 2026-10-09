import { ArchitectureDiagram } from "@/src/components/ArchitectureDiagramPage";
import { ArchitectureSection } from "@/src/components/ArchitectureSection";
import { FeatureSection } from "@/src/components/FeatureSection";
import { Footer } from "@/src/components/Footer";
import { Hero } from "@/src/components/Hero";
import { LenisProvider } from "@/src/components/LenisProvider";
import { Navbar } from "@/src/components/Navbar";
import { Portal } from "@/src/components/Motion";
import { ScrollCanvas } from "@/src/components/ScrollCanvas";
import { ScrollChoreography } from "@/src/components/ScrollChoreography";
import { SectionTransition } from "@/src/components/SectionTransition";
import { TechStackSection } from "@/src/components/TechStackSection";

export default function Home() {
  return (
    <main id="main-content" className="relative page-sections">
      <LenisProvider />
      <ScrollChoreography />
      <ScrollCanvas />
      <Navbar />
      <SectionTransition><Hero /></SectionTransition>
      <SectionTransition><FeatureSection /></SectionTransition>
      <SectionTransition><ArchitectureSection /></SectionTransition>
      <SectionTransition><TechStackSection /></SectionTransition>
      <ArchitectureDiagram embedded />
      <SectionTransition><Portal /></SectionTransition>
      <Footer />
    </main>
  );
}
