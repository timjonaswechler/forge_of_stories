Hier ist eine übersichtliche Zusammenfassung der empfohlenen Kameraeinstellungen für **Third Person** und **Pan Orbit** Kameras in Spielen, basierend auf etablierten technischen Richtlinien und typischen Spielimplementierungen :[1][11][12][13]

### Third Person Kamera

| Parameter | Beschreibung | Empfohlene Werte | Hinweise |
|------------|---------------|------------------|-----------|
| **Distance (Abstand)** | Abstand der Kamera hinter dem Spieler | 3–8 Einheiten | Näher für Shooter (3–4), weiter für Adventure (5–7) |
| **Pitch (Vertikale Neigung)** | Blickwinkel vertikal | -45° bis +60° | Standard um 15–25° leicht über horizontaler Achse |
| **Yaw (Horizontale Rotation)** | Rotation um Spieler | 0–360° | Auto-Reset auf 0° nach Inaktivität |
| **Lag Speed (Verzögerung)** | Smoothing zwischen Bewegung und Position | 2–10 | Niedrig = weich, hoch = reaktionsschnell |
| **Spring Arm Dämpfung** | Kamera über einen Feder-Arm fixiert | Federkonstante 50–200 | Kritisch gedämpft für natürliche Bewegung |
| **Collision Handling** | Wand- und Objektinteraktion | Clip-Plane 0.1–0.3, Kameragröße 0.2–0.5 | Reduziert Clipping über Distance-Reduktion |
| **Framing Offset** | Spieler-Position im Bildrahmen | (0, -0.3) | Bottom-third framing für gute Sicht auf Umgebung |
| **Auto-Reset Delay** | Zeit bis Blickrichtung zurückkehrt | 2–5 Sekunden | Sanfter Übergang bei Inaktivität |

### Pan Orbit Kamera

| Parameter | Beschreibung | Empfohlene Werte | Hinweise |
|------------|---------------|------------------|-----------|
| **Orbit Radius** | Basisabstand vom Drehpunkt | 5–15 Einheiten | Kleinere Szenen 2–5, große 10–20 |
| **Rotation Sensitivity** | Reaktionsgeschwindigkeit der Kamera | Horizontal: 2–5, Vertikal: 1–3 | Vertikal weniger empfindlich für Kontrolle |
| **Zoom Min/Max Distance** | Näherungsgrenzen der Kamera | 1–2 min, 20–50 max | Schützt vor Clipping oder zu großer Distanz |
| **Zoom Speed** | Zoomgeschwindigkeit | 1–3 | Geringere Werte = kontrollierter Zoom |
| **Smoothing Factor** | Bewegungsdämpfung | 0.8 | Weiches, natürliches Kamera-Feeling |
| **Pan Sensitivity** | Bewegung bei Tasteneingabe | 0.001 | Entspricht ~1000 Pixel pro Welteinheit |
| **Orbit Sensitivity** | Grad pro Pixel Mausbewegung | 0.1°/px (0.00175 rad) | Gleicht typische 3D-Editor-Erfahrung aus |
| **Zoom Sensitivity** | Logarithmische Zoom-Stärke | 0.01 | Exponentielles Zoom-Verhalten für besseres Gefühl |

### Zusatzfunktionen
- **Sweet Spot Zone:** Bereich (10–20 % des Bildschirms), in dem kleine Spielerbewegungen keine Kamerabewegung auslösen[11]
- **Auto-Framing:** Automatische Anpassung bei Kollisionen oder Sichtblockaden[12]
- **Debug-Hilfen:** Sichtbare Kollisionszonen und Frustum-Overlays zur Feineinstellung[11]

Diese Kombination sorgt für eine natürliche, reaktive Kamerasteuerung, wie sie in modernen Spielen wie *Horizon: Zero Dawn* oder *Zelda: Breath of the Wild* realisiert ist.[12][11]

[1](https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html)
[2](https://help.autodesk.com/view/3DSMAX/2025/ENU/?guid=GUID-2937D91D-FC36-4078-B11D-AE42F3AAB964)
[3](https://threejs.org/docs/)
[4](https://www.reddit.com/r/groundbranch/comments/pzgg6q/adjust_camera_3rd_person1st_person_view_angle/)
[5](https://www.reddit.com/r/ffxiv/comments/13g5oqt/3rd_person_camera_angle_suggestion/)
[6](https://docs.godotengine.org/de/4.x/tutorials/3d/introduction_to_3d.html)
[7](https://www.fe-lexikon.info/material/tutorials/kallenrode_os/fernerkundung-master-small.pdf)
[8](https://download.autodesk.com/us/support/files/autocad_architecture_2011_user_guide/autocad_aca_user_guide_german.pdf)
[9](https://dewesoft.com/de/blog/dewesoft-x3-sp7-freigegeben)
[10](https://at.pinterest.com/pin/682999099726581042/)
[11](http://www.gameaipro.com/GameAIPro/GameAIPro_Chapter47_Tips_and_Tricks_for_a_Robust_Third-Person_Camera_System.pdf)
[12](https://blog.littlepolygon.com/posts/cameras/)
[13](https://techarthub.com/orbit-camera-docs/)


Hier ist eine strukturierte Übersicht empfohlener Kameraeinstellungen für **First Person (Ego-Perspektive)** in Spielen. Diese Werte basieren auf Erkenntnissen aus professionellen Game-Design-Ressourcen und Entwicklerpraxis.[1][2][3][4][5][6]

### First Person Kamera

| Parameter | Beschreibung | Empfohlene Werte | Hinweise |
|------------|---------------|------------------|-----------|
| **Field of View (FOV)** | Sichtfeld, bestimmt Raumgefühl und Geschwindigkeitsempfinden | 75–100° (Standard 90°) | Niedriger Wert für enge Räume, höher für weite Szenen [1][7] |
| **Camera Height Offset** | Position relativ zum Charakterkopf | +0.6–+0.8 m über Boden | Entspricht typischer Augenhöhe der Spielfigur [1] |
| **Head Offset (Z)** | Kamera-Versatz von Kopfknochen | 0.05–0.15 m nach vorne | Reduziert Clipping mit eigener Geometrie [3][5] |
| **Pitch Limit (Vertikalrotation)** | Begrenzung des Blickwinkels nach oben/unten | -85° bis +85° | Verhindert unnatürliche Kopfrotation [1][2] |
| **Yaw Speed** | Rotationsgeschwindigkeit | 90–180°/s |  Schnellere Werte für Shooter, langsamere für Exploration [2] |
| **Mouse Sensitivity** | Empfindlichkeit der Maussteuerung | 0.4–0.8 | Werte < 0.5 geben mehr Präzision; 0.6 Standard [5][8] |
| **Smoothing Factor** | Übergangsgeschwindigkeit bei Rotation | 0.05–0.2 | Höher = glatte Kamera, niedriger = direkter Input [4][5] |
| **Rotation Lag Speed** | Verzögerung für natürliches Nachziehen | 5–15 | Bewirkt realistisches „Gewicht“ in der Bewegung [2][4] |
| **Head Bob Intensity** | Stärke der Kopfbewegung beim Laufen | 0.01–0.03 Einheiten | Zu hohe Werte wirken künstlich; optional deaktivierbar [3] |
| **ADS (Aim Down Sights) FOV** | Verengtes Sichtfeld beim Zielen | 60–70° | Verstärkt das Zielgefühl und Bewegungsfokus [9] |
| **Camera Collision** | Clipping-Schutz bei engen Räumen | Radius 0.2–0.4 | Für Head Clipping mit Geometrie wichtig [3] |

### Erweiterte Features

- **True First Person Setup:** Kamera folgt Kopfbewegung, aber mit Dämpfung, um Head-Bob-Effekte zu kontrollieren.[3]
- **Camera Lag per Rotation Spring:** Verwendung eines *SpringArm* mit `Lag Speed` zwischen 5 und 20 für flüssige Bewegung.[2]
- **Body Awareness:** Spieler sieht Teile des eigenen Körpers (Hände, Unterarme, Beine) für mehr Immersion.[10][3]
- **Animation Sync:** Waffen- oder Handanimationen über Blendspace mit Kamera-Pitch synchronisieren.[3]
- **Stabilized ADS:** Für präzises Zielen separate FOV-Interpolation (über 0.15–0.3 Sekunden Übergangszeit).[9]

Diese Kombination ergibt eine reaktive, immersive und technisch saubere First-Person-Kamera, wie man sie aus Spielen wie *Half-Life: Alyx*, *Escape from Tarkov* oder *Call of Duty* kennt.[10][9]

[1](https://learn.hypehype.com/art-and-assets/working-with-cameras/how-to-make-a-first-person-game)
[2](https://www.youtube.com/watch?v=Wbf9E0xOYmU)
[3](https://www.gamedev.net/forums/topic/669278-ways-to-setup-true-first-person-viewpoint/)
[4](https://www.youtube.com/watch?v=OS8tHtfGn-M)
[5](https://devforum.roblox.com/t/smooth-first-person-camera/1980622)
[6](https://playtank.io/2023/05/12/first-person-3cs-camera/)
[7](https://www.neogaf.com/threads/whats-the-best-fov-settings-for-1st-person-and-3rd-person-games.1630526/)
[8](https://www.reddit.com/r/Unity3D/comments/1jmau60/first_person_controller_mouse_sensitivity/)
[9](https://www.pcgamer.com/games/fps/i-tried-that-viral-battlefield-6-mouse-tweak-that-makes-sensitivity-more-consistent-and-yep-my-aim-already-feels-smoother/)
[10](https://michelsabbagh.wordpress.com/2016/03/10/making-the-most-out-of-the-first-person-perspective/)
[11](https://www.reddit.com/r/gamedev/comments/1cn56ne/first_person_camera_principles_for_beginners/)
[12](https://www.youtube.com/watch?v=Y9Eo9iSdwz4)
[13](https://www.youtube.com/watch?v=0CDvSM9kEpU)
[14](https://forums.coregames.com/t/changing-the-firstperson-view/1042)
[15](https://www.youtube.com/watch?v=2-LrYMD9V08)
[16](https://forums.unrealengine.com/t/first-person-camera-creation/375717)
[17](https://www.youtube.com/watch?v=CvRhYrBMsKI)
[18](https://devforum.roblox.com/t/how-would-i-go-about-making-a-smooth-custom-first-person-camera-system-like-doors/2200738)
[19](https://adventurecreator.org/forum/discussion/15720/smooth-moving-transitions-in-first-person-mode)


mögliche TODOS:

- Defaults aus ReadCamera.md noch erweitern: z. B. FOV/ADS, Smoothing-Faktoren.
- Eine einheitliche „CameraRig“-Entity als Tween-Ziel (vereinfachte Aktivierung).
- Optionale Ease-Kurven pro Transition (z. B. QuadraticInOut → CubicInOut).
